# Sim8051
Simulator for 8051 based microcontroller assembly 

# Build Instructions 
Make build.sh executable and execute it <br>

`chmod +x build.sh` <br> 
`sh ./build.sh` <br> <br> 

This will generate dynamic link library that will be consumed by C++ for Qt. <br>

Make sure Qt is intalled in the system. <br> 
Run cmake as : <br> 

`cmake -DCMAKE_PREFIX_PATH=<path_to_cmake_of_your_installed_qt> CMakeLists.txt`<br>

For example, in my sytem, cmake was run as <br> 
`cmake -DCMAKE_PREFIX_PATH=~/Qt/6.3.0/gcc_64/lib/cmake CMakeLists.txt` <br>
`make` <br>

Execute <br>
`./8051Sim`
