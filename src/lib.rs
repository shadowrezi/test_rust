use std::fs::File;
use std::io::Read;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use systray::Application;
use cpython::{Python, PyResult};

#[no_mangle]
pub extern fn lib_test() {
    let (tx, rx) = channel::<()>();
    let running = Arc::new(AtomicBool::new(true));
    let _tx_mutex = Arc::new(Mutex::new(tx));
    let running_clone = Arc::clone(&running);
    let mut app = Application::new().expect("Failed to create application");

    app.add_menu_item("Hello", |_| {
        println!("Hello from menu!");
        Ok::<(), systray::Error>(())
    }).expect("Failed to add menu item");

    app.add_menu_item("ChatGPT TG", move |_| {
        turn_on_bot(running_clone.clone());
        Ok::<(), systray::Error>(())
    }).expect("Failed to add start menu item");

    app.add_menu_item("Stop Bot", move |_| {
        stop_bot(&mut running.clone());
        Ok::<(), systray::Error>(())
    }).expect("Failed to add stop menu item");

    thread::spawn(move || {
        let _ = app.wait_for_message();
    });

    rx.recv().expect("Failed to receive signal for exit");
}

fn turn_on_bot(running: Arc<AtomicBool>) {
    thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            let gil = Python::acquire_gil();
            let py = gil.python();

            let code = read_file_contents(
                r"C:\Users\Administrator\Documents\Python\binary\telegram_bot.py"
            );
            
            match code {
                Ok(code_content) => {
                    let result: PyResult<()> = py.run(&code_content, None, None);
                    match result {
                        Ok(_) => {},
                        Err(err) => eprintln!("Python code execution error: {:?}", err),
                    }
                }
                Err(err) => eprintln!("Failed to read file: {}", err),
            }
        }
    });
}

fn stop_bot(running: &mut Arc<AtomicBool>) {
    running.store(false, Ordering::Relaxed);
}

fn read_file_contents(file_path: &str) -> std::io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
