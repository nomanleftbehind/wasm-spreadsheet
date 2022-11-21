use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

use crate::console_log as log;
mod utils;

#[derive(Clone, Debug, PartialEq)]
pub enum CellValue {
    String(Option<String>),
    Int(Option<i32>),
    Float(Option<f32>),
}

#[derive(Debug)]
pub struct CellObject<'a> {
    pub ctx: &'a CanvasRenderingContext2d,
    pub column_id: u32,
    pub row_id: u32,
    height: f64,
    width: f64,
    value: CellValue,
}

impl<'a> CellObject<'a> {
    pub fn new(
        ctx: &'a CanvasRenderingContext2d,
        column_id: u32,
        row_id: u32,
        height: f64,
        width: f64,
    ) -> Self {
        utils::set_panic_hook();

        let value = CellValue::String(None);

        Self {
            ctx,
            column_id,
            row_id,
            value,
            height,
            width,
        }
    }

    pub fn get_value(&self) -> CellValue {
        self.value.clone()
    }
}

pub trait Border {
    fn draw(&self);
}

impl<'a> Border for CellObject<'a> {
    fn draw(&self) {
        let CellObject {
            ctx,
            column_id,
            row_id,
            height,
            width,
            ..
        } = self;
        ctx.begin_path();
        ctx.rect(
            *column_id as f64 * width,
            *row_id as f64 * height,
            *width,
            *height,
        );
        ctx.stroke();
    }
}

#[derive(Debug)]
pub enum ColumnType {
    String,
    Int,
    Float,
}

#[derive(Debug)]
pub struct Column<'a> {
    pub column_id: u32,
    pub column_type: ColumnType,
    pub cells: Vec<CellObject<'a>>,
    width: f64,
}

impl<'a> Column<'a> {
    pub fn new(
        ctx: &'a CanvasRenderingContext2d,
        column_id: u32,
        num_rows: u32,
        width: f64,
    ) -> Self {
        utils::set_panic_hook();

        let column_type = ColumnType::String;
        let cells = (0..num_rows)
            .map(|row_id| {
                let cell = CellObject::new(ctx, column_id, row_id, 30.0, width);
                cell.draw();
                cell
            })
            .collect();

        Self {
            column_id,
            column_type,
            width,
            cells,
        }
    }

    fn get_width(&self) -> f64 {
        self.width
    }

    fn set_width(&mut self, new_width: f64) {
        self.width = new_width;
    }

    fn get_column_name(&self) -> String {
        let mut n = self.column_id + 1;
        let mut name = String::new();

        while n > 0 {
            n = n - 1;
            let k = n % 26;
            if let Some((_, c)) = (b'A'..=b'Z').enumerate().find(|&(i, _)| i as u32 == k) {
                name.push(c as char);
            }
            n = n / 26;
        }
        name.chars().rev().collect()
    }
}

#[derive(Debug)]
pub struct Grid<'a> {
    num_rows: u32,
    num_cols: u32,
    columns: Vec<Column<'a>>,
}

impl<'a> Grid<'a> {
    pub fn new(ctx: &'a CanvasRenderingContext2d, num_rows: u32, num_cols: u32) -> Self {
        utils::set_panic_hook();

        let columns = (0..num_cols)
            .map(|column_id| Column::new(ctx, column_id, num_rows, 80.0))
            .collect();

        Self {
            columns,
            num_rows,
            num_cols,
        }
    }

    pub fn get_column(&self, col_num: u32) -> Option<&Column> {
        self.columns.get(col_num as usize)
    }

    pub fn get_width(&self) -> f64 {
        self.columns.iter().map(|col| col.get_width()).sum()
    }
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    body.append_child(&canvas)?;

    let width = window.inner_width()?.as_f64().unwrap_or_else(|| 640.0) as u32;
    let height = window.inner_height()?.as_f64().unwrap_or_else(|| 480.0) as u32;

    canvas.set_width(width);
    canvas.set_height(height);
    canvas.style().set_property("border", "solid red")?;
    // canvas.style().set_property("margin", "100px")?;
    // canvas.style().set_property("width", "100%")?;
    // canvas.style().set_property("height", "100%")?;
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let grid = Grid::new(&context, 12, 350);

    let a = grid.get_width();
    // canvas.set_width(a as u32);

    log!("grid width: {}", a);
    let col = grid.get_column(349);

    if let Some(c) = col {
        let fd = c.get_column_name();
        log!("{}", fd);
    }

    let context = Rc::new(context);
    let pressed = Rc::new(Cell::new(false));
    {
        let context = context.clone();
        let pressed = pressed.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            context.begin_path();
            context.move_to(event.offset_x() as f64, event.offset_y() as f64);
            pressed.set(true);
        });
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let context = context.clone();
        let pressed = pressed.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            if pressed.get() {
                context.line_to(event.offset_x() as f64, event.offset_y() as f64);
                context.stroke();
                context.begin_path();
                context.move_to(event.offset_x() as f64, event.offset_y() as f64);
            }
        });
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let context = context.clone();
        let pressed = pressed.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
            pressed.set(false);
            context.line_to(event.offset_x() as f64, event.offset_y() as f64);
            context.stroke();
        });
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
