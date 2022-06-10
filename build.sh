#!/bin/sh 

cargo build 
cp ./target/release/libsim8051.so ./QtFrontend/lib 
