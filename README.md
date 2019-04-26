# stm32f429_disco_mandelbrot
bare metal mandelbrot implementation on stm32f429

Early stage not working yet!

After last compiler update it does not compile
because of crate.io updates!!

Trying to take this:
https://github.com/adamgreig/stm32f4-smoltcp-demo



Prerequisit:

 rust stable is required 
 
 rustup target add  thumbv7em-none-eabihf
 
 openocd
 
 gdb-multiarch
 
 

build with:
>> cargo build

flash with:
>> . ./openocd_program.sh target/thumbv7em-none-eabihf/debug/mandelbrot
