#! /usr/bin/bash

# Start with SEAL-Python
cd seal;

# Install dependencies
pip3 install numpy pybind11 networkx pygraphviz

# Init the SEAL and pybind11
git submodule update --init --recursive

# Build the SEAL lib
cd SEAL;
cmake -S . -B build -DSEAL_USE_MSGSL=OFF -DSEAL_USE_ZLIB=OFF;
cmake --build build;
cd ..;

# Run the setup.py
python3 setup.py build_ext -i

# Copy the Lib to the src-directory
cp seal.*.so ../src