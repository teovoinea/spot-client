extern crate stdweb;

use std::rc::Rc;

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::{
    HtmlElement,
    document,
    window,
    CanvasRenderingContext2d,
    WebSocket,
};

use stdweb::web::event::{
    MouseMoveEvent,
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

    let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let context: CanvasRenderingContext2d = canvas.get_context().unwrap();

    canvas.set_width(canvas.offset_width() as u32);
    canvas.set_height(canvas.offset_height() as u32);

    // window().add_event_listener( enclose!( (canvas) move |_: ResizeEvent| {
    //     canvas.set_width(canvas.offset_width() as u32);
    //     canvas.set_height(canvas.offset_height() as u32);
    // }));

    canvas.add_event_listener( enclose!( (context) move |event: MouseMoveEvent| {
        context.fill_rect(f64::from(event.client_x() - 5), f64::from(event.client_y() - 5)
                          , 10.0, 10.0);
    }));


    output_msg("> Connecting...");

    let ws = WebSocket::new_with_protocols("ws://localhost:2794", &["rust-websocket"]).unwrap();

    ws.add_event_listener( enclose!( (output_msg) move |_: SocketOpenEvent| {
        output_msg("> Opened connection");
    }));

    ws.add_event_listener( enclose!( (output_msg) move |_: SocketErrorEvent| {
        output_msg("> Connection Errored");
    }));

    ws.add_event_listener( enclose!( (output_msg) move |event: SocketCloseEvent| {
        output_msg(&format!("> Connection Closed: {}", event.reason()));
    }));

    ws.add_event_listener( enclose!( (output_msg) move |event: SocketMessageEvent| {
        output_msg(&event.data().into_text().unwrap());
    }));

    let text_entry: InputElement = document().query_selector( ".form input" ).unwrap().unwrap().try_into().unwrap();
    text_entry.add_event_listener( enclose!( (text_entry) move |event: KeyPressEvent| {
        if event.key() == "Enter" {
            event.prevent_default();

            let text: String = text_entry.raw_value();
            if text.is_empty() == false {
                text_entry.set_raw_value("");
                ws.send_text(&text).unwrap();
            }
        }
    }));


    stdweb::event_loop();
}
