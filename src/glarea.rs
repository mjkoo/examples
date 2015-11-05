//! # GLArea Sample
//!
//! This sample demonstrates how to use GLAreas and OpenGL

// make moving clones into closures more convenient
macro_rules! clone {
    ($($n:ident),+; || $body:block) => (
        {
            $( let $n = $n.clone(); )+
            move || { $body }
        }
    );
    ($($n:ident),+; |$($p:ident),+| $body:block) => (
        {
            $( let $n = $n.clone(); )+
            move |$($p),+| { $body }
        }
    );
}

#[cfg(feature = "opengl")]
mod example {
    extern crate gtk;
    extern crate libc;
    extern crate glium;
    extern crate epoxy;
    extern crate shared_library;

    use std::ptr;
    use std::cell::RefCell;
    use std::rc::Rc;

    use self::gtk::traits::*;
    use self::gtk::signal::Inhibit;
    use self::gtk::{GLArea, Window};

    use self::glium::Surface;

    use self::shared_library::dynamic_library::DynamicLibrary;

    pub fn main() {
        if gtk::init().is_err() {
            println!("Failed to initialize GTK.");
            return;
        }

        let window = Window::new(gtk::WindowType::Toplevel).unwrap();
        let glarea = GLArea::new().unwrap();

        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        epoxy::load_with(|s| {
            unsafe {
                match DynamicLibrary::open(None).unwrap().symbol(s) {
                    Ok(v) => v,
                    Err(_) => ptr::null(),
                }
            }
        });

        struct Backend {
            glarea: GLArea,
        }

        unsafe impl glium::backend::Backend for Backend {
            fn swap_buffers(&self) -> Result<(), glium::SwapBuffersError> {
                self.glarea.queue_render();
                Ok(())
            }

            unsafe fn get_proc_address(&self, symbol: &str) -> *const libc::c_void {
                epoxy::get_proc_addr(symbol)
            }

            fn get_framebuffer_dimensions(&self) -> (u32, u32) {
                (self.glarea.get_allocated_width() as u32, self.glarea.get_allocated_height() as u32)
            }

            fn is_current(&self) -> bool {
                unsafe { self.make_current() };
                true
            }

            unsafe fn make_current(&self) {
                if self.glarea.get_realized() {
                    self.glarea.make_current();
                }
            }
        }

        let context: Rc<RefCell<Option<Rc<glium::backend::Context>>>> = Rc::new(RefCell::new(None));
        glarea.connect_realize(clone!(glarea, context; |_widget| {
            let mut context = context.borrow_mut();
            *context = Some(unsafe {
                glium::backend::Context::new::<_, ()>(Backend { glarea: glarea.clone() },
                    true, Default::default())
            }.unwrap());
        }));

        glarea.connect_render(clone!(context; |_glarea, _glctx| {
            let context = context.borrow();
            let mut target = glium::Frame::new(context.as_ref().unwrap().clone(),
                context.as_ref().unwrap().get_framebuffer_dimensions());
            target.clear_color(1.0, 0.0, 0.0, 1.0);
            target.finish().unwrap();

            Inhibit(false)
        }));

        window.set_title("GLArea Example");
        window.set_default_size(400, 400);
        window.add(&glarea);

        window.show_all();
        gtk::main();
    }
}

#[cfg(feature = "opengl")]
fn main() {
    example::main()
}

#[cfg(not(feature = "opengl"))]
fn main() {
    println!("Did you forget to build with `--features opengl`?");
}
