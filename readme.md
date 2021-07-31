# Rust mTLS example

This is an example of how to support mTLS in Rust. Implementation includes both the client and the server side.

- Server: [warp](https://github.com/seanmonstar/warp)
- Client: [reqwest](https://github.com/seanmonstar/reqwest) (with `native-tls`)

To run the example:

1. Generate all required keys and certificates following [the instructions below](#setup-your-own-certificate-authority).
2. Use `cd ..` to go back to the project root folder.
3. Start the server by running `cargo run -- server` (note the white space).
4. (While the server is running) start another prompt and run `cargo run -- client` (note the white space). 
5. If you see `"Hello, mTLS World!"`, your configuration is probably correct. Congratulations!

Requirements:

- Rust & Cargo
- openssl (>=2019)
- *(optional)* curl (>=2019)

Notes:

- To use this in a real-world scenario, please replace `localhost` with a meaningful domain name. Do not forget to also change it in the SAN (subjectAltName). Also do not forget to fill in meaningful information while generating CSRs.

- To test the mTLS server in your browser (e.g. Chrome), please see [this nice post](https://blog.eldernode.com/install-root-certificate-in-chrome/) or search "manual install certification {your browser name}". However, please note that
    - To make the https work (make the small lock icon green), the `localhost.bundle.crt` mentioned below need to be installed in the `trusted root certifications` category.
    - To make the client authentication work, just import the client's `client_0.p12` file mentioned below into `Personal` category. In the import window, you might need to change the extension name to find the right file.

## Setup your own Certificate Authority

### CA

1. Create a folder for all related files
    
    ```sh
    mkdir ca
    cd ca
    ```

2. Generate a key for the CA

    ```sh
    openssl genrsa -out ca.key 2048
    ```

3. Generate a self signed certificate for the CA

    ```sh
    # enter detailed information when necessary
    openssl req -new -x509 -key ca.key -out ca.crt
    ```

### Server

4. Generate an RSA key for the domain (`localhost` here)

    ```sh
    openssl genrsa -out localhost.key 2048
    # optional: inspect the key
    openssl rsa -in localhost.key -noout -text
    # optional: extract pubkey
    openssl rsa -in localhost.key -pubout -out localhost.pubkey
    ```

5. Generate a Certificate Signing Request (CSR)

    ```sh
    # enter detailed information when necessary (please make sure you enter COMMON NAME)
    openssl req -new -key localhost.key -addext "subjectAltName = DNS:localhost" -out localhost.csr
    # optional: inspect the csr (note: while inspecting, make sure your Signature Algorithm is not MD5 which is not accepted by many sites, upgrade your openssl if necessary)
    openssl req -in localhost.csr -noout -text
    ```

6. Sign the domain certificate 

    ```sh
    openssl x509 -req -in localhost.csr -CA ca.crt -CAkey ca.key -CAcreateserial -extfile <(printf "subjectAltName=DNS:localhost") -out localhost.crt
    # optional: to exam the output crt
    openssl x509 -in localhost.crt -noout -text
    ```

7. Create another file that contains the domain certificate and the ca certificate

    ```sh
    cat localhost.crt ca.crt > localhost.bundle.crt
    ```

### Client

8. Generate an RSA key for the client

    ```sh
    openssl genrsa -out client_0.key 2048
    ```

9. Generate a Certificate Signing Request (CSR), please note that the SAN extension is NECESSARY 

    ```sh
    # enter detailed information when necessary (please make sure you enter COMMON NAME)
    openssl req -new -key client_0.key -addext "subjectAltName = DNS:localhost" -out client_0.csr
    ```

10. Use CA key to sign it

    ```sh
    openssl x509 -req -in client_0.csr -CA ca.crt -CAkey ca.key -CAcreateserial -extfile <(printf "subjectAltName=DNS:localhost") -out client_0.crt
    ```

11. Generate pem file to test with curl & browser

    ```sh
    # generate pem file
    cat client_0.crt client_0.key > client_0.pem
    # optional: test command (after starting the server) using .pem file
    curl -L  https://localhost:3030/ --cacert ca.crt --cert client_0.pem -v
    # generate cert file to use with browser (setting password to be 123456 for example)
    openssl pkcs12 -export -in client_0.pem -out client_0.p12 -name "client_0"
    # optional: test command (after starting the server) using .p12 file
    curl -L  https://localhost:3030/ --cacert ca.crt --cert-type P12 --cert client_0.p12:123456 -v
    ```

### Optional - create certificates for more clients

12. Instructions are the same as above

    ```sh
    export CLIENT_NAME=client_1
    openssl genrsa -out ${CLIENT_NAME}.key 2048
    openssl req -new -key ${CLIENT_NAME}.key -addext "subjectAltName = DNS:localhost" -out ${CLIENT_NAME}.csr
    ```
    ```sh
    # after answering the prompt above
    openssl x509 -req -in ${CLIENT_NAME}.csr -CA ca.crt -CAkey ca.key -CAcreateserial -extfile <(printf "subjectAltName=DNS:localhost") -out ${CLIENT_NAME}.crt
    cat ${CLIENT_NAME}.crt ${CLIENT_NAME}.key > ${CLIENT_NAME}.pem
    ```

## Reference: 

- A **SUPER** useful gist: https://gist.github.com/Soarez/9688998
- A Rust mTLS example using openssl and Actix: https://github.com/sjolicoeur/rust-mtls-example-server/blob/master/bin/create_certs.sh
- OpenSSL official doc: https://www.openssl.org/docs/man1.1.1/man1/
- Reqwest doc related to TLS client authentication: https://docs.rs/reqwest/0.11.4/reqwest/struct.ClientBuilder.html#method.identity
- Warp doc related to TLS client authentication: https://docs.rs/warp/0.3.1/warp/struct.TlsServer.html#method.client_auth_required_path

## Contributing

There are still a few unclear issues, including:

- Why the code does not work with `rustls-tls` feature using `.pem` Identity?
- Why the certificate for the client must include the extension fields of SAN?

If you know the answers or have better solutions, please feel free to share your thoughts or send issues/PRs. Contributions are greatly appreciated. 
