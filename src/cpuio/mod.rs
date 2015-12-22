mod port;

pub use self::port::{Port, UnsafePort};
use cpuid::*;


pub struct Pic {
    offset: u8,
    command: UnsafePort<u8>,
    data: UnsafePort<u8>
}


impl Pic {
    pub fn handles_interrupt() {}
}

pub fn setup_apic() {
    if !get_features().contains(APIC) {
        panic!("No APIC support found");
    }

    set_color!(LIGHT_GRAY);
    print!("{:73}", "Setting up APIC");

    disable_8259_pic();




    set_color!(WHITE);
    print!("[");
    set_color!(GREEN);
    print!(" OK ");
    set_color!(WHITE);
    println!("] ");

}

fn disable_8259_pic() {
    unsafe {
        let pic1 = UnsafePort::<u8>::new(0x21);
        pic1.write(0xff);

        let pic2 = UnsafePort::<u8>::new(0xa1);
        pic2.write(0xff);
    }
}