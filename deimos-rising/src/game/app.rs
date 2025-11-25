use std::fmt::Write;

use memory::Mem;
use win32_winapi::calling_convention::FromStack;

pub fn app_log(arg: Option<&str>, (mem, esp): (Mem, u32)) {
    let arg = arg.unwrap();

    let mut result = String::new();
    let mut arg_index = 0;
    let mut chars = arg.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next_ch) = chars.peek() {
                match next_ch {
                    's' => {
                        chars.next(); // consume 's'
                        let stack_arg = <Option<&str>>::from_stack(mem, esp + arg_index * 4);
                        if let Some(s) = stack_arg {
                            result.push_str(s);
                        } else {
                            result.push_str("(null)");
                        }
                        arg_index += 1;
                    }
                    'i' => {
                        chars.next(); // consume 'i'
                        write!(result, "{}", i32::from_stack(mem, esp + arg_index * 4)).unwrap();
                        arg_index += 1;
                    }
                    _ => {
                        result.push(ch);
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    eprintln!("{}", result);
}
