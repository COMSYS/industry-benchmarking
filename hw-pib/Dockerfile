# Use rust compiler with scone
FROM registry.scontain.com:5050/sconecuratedimages/crosscompilers:ubuntu

# Copy the code from the repo to /usr/src/NAME
COPY ./impl /usr/src/teebench

# Set workdir to make code compile
WORKDIR /usr/src/teebench/server

# Create data directory - for writing
RUN mkdir -p ../data/server_data

# Install rust program in docker container
RUN scone-cargo install --features="evaluation" --target=x86_64-scone-linux-musl --path .

# Add new path to /root/.cargo/bin
ENV PATH "$PATH:/root/.cargo/bin"
RUN echo "export PATH=${PATH}:/root/.cargo/bin" >> /root/.bashrc

# Copy over entry script which gets executed on run
COPY ./container-scripts/execute.sh /

# Make script executable
RUN chmod +x /execute.sh

# Run calculation based on given input
ENTRYPOINT ["/execute.sh"]
