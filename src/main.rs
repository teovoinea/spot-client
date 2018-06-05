#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::u8::MAX;
use std::i64;

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
    CanvasStyle,
};

use stdweb::web::event::{
    ChangeEvent,
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

    let output_msg = Rc::new(move |msg: &str| {
        js!( @(no_return)
            console.log(@{msg});
        );
    });
    output_msg("> Connecting...");

    let ws = WebSocket::new("ws://localhost:2794").unwrap();
    ws.set_binary_type(SocketBinaryType::ArrayBuffer);

    let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let paint_color: HtmlElement = document().query_selector( "#paint-color" ).unwrap().unwrap().try_into().unwrap();
    let context: CanvasRenderingContext2d = canvas.get_context().unwrap();
    let left: i32 = js! (
        return canvas.offsetLeft;
    ).try_into().unwrap();
    let top: i32 = js! (
        return canvas.offsetTop;
    ).try_into().unwrap();

    canvas.set_width(640 as u32);
    canvas.set_height(480 as u32);

    paint_color.add_event_listener( enclose!((paint_color, context) move |_: ChangeEvent| {
        let color: String = js! (
            return @{paint_color.as_ref()}.value;
        ).try_into().unwrap();
        let mut hex_color: String = String::from("#");
        hex_color.push_str(&color);
        context.set_fill_style_color(&hex_color);
    }));

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
        let mut hex_color: String = String::from("#");
        hex_color.push_str(&format!("{:X}", decoded_pixel.r));
        hex_color.push_str(&format!("{:X}", decoded_pixel.g));
        hex_color.push_str(&format!("{:X}", decoded_pixel.b));

        let old_color: String = match context.get_fill_style() {
            CanvasStyle::String(s) => {
                s
            },
            _ => {
                String::from("#000000")
            }
        };

        context.set_fill_style_color(&hex_color);
        context.fill_rect(f64::from(decoded_pixel.x - 5), f64::from(decoded_pixel.y - 5)
                    , 10.0, 10.0);
        context.set_fill_style_color(&old_color);
        output_msg(&format!("Received pixel: {:?}", decoded_pixel));
    }));

    canvas.add_event_listener( enclose!( (context, mouse_down, left) move |event: MouseMoveEvent| {
        if mouse_down.load(Ordering::Relaxed) {
            let x = event.client_x() - left;
            let y = event.client_y() - top;
            context.fill_rect(f64::from(x), f64::from(y)
                    , 10.0, 10.0);
            let color: String = match context.get_fill_style() {
                CanvasStyle::String(s) => {
                    s
                },
                _ => {
                    String::from("#000000")
                }
            };

            //js! ( @(no_return)
            //    console.log(@{&x}, @{&y});
            //);

            let r = i64::from_str_radix(&color[1..3], 16).unwrap();
            let g = i64::from_str_radix(&color[3..5], 16).unwrap();
            let b = i64::from_str_radix(&color[5..7], 16).unwrap();

            let current_pixel = PaintPixel {
                x: x,
                y: y,
                r: r as u8,
                g: g as u8,
                b: b as u8,
            };
            let ser: Vec<u8> = serialize(&current_pixel).unwrap();
            let ab: ArrayBuffer = TypedArray::from(ser.as_slice()).buffer();
            ws.send_array_buffer(&ab).unwrap();
        }
    }));

    stdweb::event_loop();
}
