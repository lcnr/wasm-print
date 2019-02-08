#![feature(set_stdio)]

use std::panic;
use std::io;

use wasm_bindgen::prelude::*;

fn print(buf: &str) -> io::Result<()> {
    web_sys::console::info_1(&JsValue::from(buf));
    Ok(())
}

fn eprint(buf: &str) -> io::Result<()> {
    web_sys::console::warn_1(&JsValue::from(buf));
    Ok(())
}


struct Printer<T: FnMut(&str) -> io::Result<()>> {
    print_fn: T,
    buffer: String,
    is_buffered: bool,
}

impl<T: FnMut(&str) -> io::Result<()>> Printer<T> {
    // return box for the sake of simplicity
    fn new(print_fn: T, is_buffered: bool) -> Box<Printer<T>> {
        Box::new(Printer {
            buffer: String::new(),
            print_fn,
            is_buffered,
        })
    }
}

impl<T: FnMut(&str) -> io::Result<()>> io::Write for Printer<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.push_str(&String::from_utf8_lossy(buf));

        if !self.is_buffered {
            (self.print_fn)(&self.buffer)?;
            self.buffer.clear();
        }
        else if let Some(i) = self.buffer.rfind('\n') {
            let buffered = {
                let (first, last) = self.buffer.split_at(i);
                (self.print_fn)(first)?;

                String::from(&last[1..])
            };

            self.buffer.clear();
            self.buffer.push_str(&buffered);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() { 
            (self.print_fn)(&self.buffer)?;
            self.buffer.clear();
        }
        Ok(())
    }
}


pub fn set_stdout() {
    io::set_print(Some(Printer::new(print, true)));
}

pub fn set_stdout_unbuffered() {
    io::set_print(Some(Printer::new(print, false)));
}

pub fn set_stderr() {
    io::set_panic(Some(Printer::new(eprint, true)));
}

pub fn set_stderr_unbuffered() {
    io::set_panic(Some(Printer::new(eprint, false)));
}

pub fn set_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "unknown location",
                }
            }
        };

        let err_info = match info.location() {
            Some(location) => {
                let file = location.file();
                let line = location.line();
                let col = location.column();
                format!("Panicked at '{}', {}:{}:{}", msg, file, line, col)
            }
            None => {
                format!("Panicked at an unknown location '{}'", msg)
            }
        };

        web_sys::console::trace_1(&JsValue::from_str(&err_info));
    }));
}

/// Sets stdout, stderr, and a custom panic hook
pub fn hook() {
    set_stdout();
    set_stderr();
    set_panic_hook();
}

/// the same as hook when called for the first time, each additional call is a noop.
#[inline(always)]
pub fn init() {
    static START: std::sync::Once = std::sync::Once::new();
    START.call_once(|| hook());
}