use std::{
    env,
    fs,
    thread,
};
use std::process::{
    Command,
    exit,
};
use std::fs::File;
use std::time::Duration;
use std::thread::sleep;
use std::io::prelude::*;
use std::process;
use std::path::Path;
use nix::unistd::Uid;

// Function to run paxctl in threads.
fn take_thread(s: String){

      let _handle = thread::spawn(move || {
          println!("Now applying paxctl to {}", &s); // Message stdout
          // Create table for paxctl:
          Command::new("paxctl") 
            .args(["-c", &s])
            .output()
            .expect("Failed to execute process!");
          // Apply flags for paxctl
          Command::new("paxctl")
            .arg("-PEMRXS")
            .arg(&s)
            .output()
            .expect("Failed to execute process!");
      });
}

fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {

    // Confirm paxctl is installed
    if !Path::new("/usr/local/sbin/paxctl").exists() {
        println!("Paxctl was not found.\nPlease install this and try again.");
        process::exit(0x0100);
    }
    
    // Confirm user is root
    if !Uid::effective().is_root() {
        println!("You must use sudo or doas to run this.\nRoot or non-root usage may be problematic.");
        process::exit(0x0100);
    }

    // Change priority on process to 15. 
    unsafe { // Set priority on this process to 15.
        libc::setpriority(libc::PRIO_PROCESS, 0, 15);
    }

    // If $PATH var not found, error.
    let path_var = match env::var("PATH") {
        Ok(val) => val,
        Err(_e) => "Failed to get $PATH variable.".to_string(),
    };

    // Replace : with newline so each directory of $PATH is on a newline.
    let output = path_var.replace(':', "\n"); // Replace : in $PATH with new lines.
    // Ask user to confirm changes
    println!("About to commit 'paxctl -PEMRXS' to all ELF binaries in $PATH directories.");
    println!("Would you like to proceed? [Y/N] ");
    
    // Get user input
    let mut line = String::new();
    let _input = std::io::stdin().read_line(&mut line).unwrap();
    
    // Taking action on Y/N input
    match line.as_str().trim() {
        "Y" | "y" => { println!("This will take some time to complete. Go make some coffee.");
        sleep(Duration::from_millis(2000)) }, // Sleep so above message is seen.
        "N" | "n" => { println!("As you wish. Exiting.");
        exit(0); },
        _ => { println!("Valid input was not received. Exiting.");
        exit(1); },
    } 
   
    // Vector to hold lists of files
    let strings: Vec<_> = output.lines().collect();

    // For each file in each directory found in $PATH:
    for a in strings {
        let mut thread_handles = vec![];
        let entries = fs::read_dir(a).unwrap();
        for entry in entries {
            // Get files in directories taken from $PATH.
            let entry = entry.unwrap();
            // If symlink, skip.
            if entry.file_type()?.is_symlink() {
                continue;
            }
            let b = entry.path().display().to_string();
            // Open file:
            let mut f = File::open(&b)?;
            let mut buffer = [0; 4];
            // If ELF binary call thread with take_thread function.
            match f.read_exact(&mut buffer) {
                Ok(()) if &buffer == b"\x7fELF" => {
                    thread_handles.push(thread::spawn(move || {
                        take_thread(b);
                    }));
                }
                Err(..) => continue,
                _ => continue,
            }
            sleep(Duration::from_millis(50)); // Added to prevent too many open files errors.
        }
        // Wait for threads and join.
        let _strings = thread_handles.into_iter().map(|h| h.join().unwrap());
    }

    // Print message on completion.
    println!("\nAll ELF binaries in $PATH have been updated.");
    println!("You may confirm with 'paxctl -v /path/to/binary'");

    Ok(())
}
