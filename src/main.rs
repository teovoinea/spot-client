extern crate stdweb;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::u8::MAX;

use bincode::{serialize, deserialize};

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::{
    HtmlElement,
    document,
    window,
    CanvasRenderingContext2d,
    WebSocket,
    ArrayBuffer,
    TypedArray,
    SocketBinaryType,
};

use stdweb::web::event::{
    MouseMoveEvent,
    MouseDownEvent,
    MouseUpEvent,
    ResizeEvent,
    KeyPressEvent,
    SocketOpenEvent,
    SocketCloseEvent,
    SocketErrorEvent,
    SocketMessageEvent,
};

use stdweb::web::html_element::{
    CanvasElement,
    InputElement,
};

// Shamelessly stolen from webplatform's TodoMVC example.
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct PaintPixel {
	x: i32,
	y: i32,
	r: u8,
	g: u8,
	b: u8,
}

fn main() {
    stdweb::initialize();

    
    let output_div: HtmlElement = document().query_selector( ".output" ).unwrap().unwrap().try_into().unwrap();
    let output_msg = Rc::new(move |msg: &str| {
        let elem = document().create_element("p").unwrap();
        elem.set_text_content(msg);
        if let Some(child) = output_div.first_child() {
            output_div.insert_before(&elem, &child).unwrap();
        } else {
            output_div.append_child(&elem);
        }
    });
    output_msg("> Connecting...");

    let ws = WebSocket::new_with_protocols("ws://localhost:2794", &["rust-websocket"]).unwrap();
    ws.set_binary_type(SocketBinaryType::ArrayBuffer);

    let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let context: CanvasRenderingContext2d = canvas.get_context().unwrap();

    canvas.set_width(640 as u32);
    canvas.set_height(480 as u32);

    window().add_event_listener( enclose!( (canvas) move |_: ResizeEvent| {
         canvas.set_width(canvas.offset_width() as u32);
         canvas.set_height(canvas.offset_height() as u32);
    }));

    let mouse_down = Arc::new(AtomicBool::new(false));

    canvas.add_event_listener( enclose!( (mouse_down) move |_: MouseDownEvent| {
        mouse_down.store(true, Ordering::Relaxed)
    }));

    canvas.add_event_listener( enclose!( (mouse_down) move |_: MouseUpEvent| {
        mouse_down.store(false, Ordering::Relaxed)
    }));

    ws.add_event_listener( enclose!( (output_msg) move |_: SocketOpenEvent| {
        output_msg("> Opened connection");
    }));

    ws.add_event_listener( enclose!( (output_msg) move |_: SocketErrorEvent| {
        output_msg("> Connection Errored");
    }));

    ws.add_event_listener( enclose!( (output_msg) move |event: SocketCloseEvent| {
        output_msg(&format!("> Connection Closed: {}", event.reason()));
    }));
    
    ws.add_event_listener( enclose!( (context, output_msg) move |event: SocketMessageEvent| {
        let data: Vec<u8> = Vec::from(event.data().into_array_buffer().unwrap());
        let decoded_pixel: PaintPixel = deserialize(&data).unwrap();
        context.fill_rect(f64::from(decoded_pixel.x - 5), f64::from(decoded_pixel.y - 5)
                    , 10.0, 10.0);
        output_msg(&format!("Received pixel: {:?}", decoded_pixel));
    }));

    canvas.add_event_listener( enclose!( (context, mouse_down) move |event: MouseMoveEvent| {
        if mouse_down.load(Ordering::Relaxed) {
            context.fill_rect(f64::from(event.client_x() - 5), f64::from(event.client_y() - 5)
                    , 10.0, 10.0);
            let current_pixel = PaintPixel {
                x: event.client_x(),
                y: event.client_y(),
                r: MAX,
                g: MAX,
                b: MAX,
            };
            let ser: Vec<u8> = serialize(&current_pixel).unwrap();
            let ab: ArrayBuffer = TypedArray::from(ser.as_slice()).buffer();
            ws.send_array_buffer(&ab).unwrap();
        }
    }));

    // let text_entry: InputElement = document().query_selector( ".form input" ).unwrap().unwrap().try_into().unwrap();
    // text_entry.add_event_listener( enclose!( (text_entry) move |event: KeyPressEvent| {
    //     if event.key() == "Enter" {
    //         event.prevent_default();

    //         let text: String = text_entry.raw_value();
    //         if text.is_empty() == false {
    //             text_entry.set_raw_value("");
    //             ws.send_text(&text).unwrap();
    //         }
    //     }
    // }));


    stdweb::event_loop();
}
