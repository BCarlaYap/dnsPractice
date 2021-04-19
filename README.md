# DNSPractice 
***

<b>prerequisite</b>: you should have docker already installed.

### running the api
1. Run `. script/start_postgres.sh`. This will run the docker postgres, where the providers list reside. Input a password. <br><br>
An `.env` file is created with the docker's ip, host and password stored. <br>
The fields of the table are:`id |    name    |               doh_url                | vote_weight`:
   * where `id` automatically increments; <br>
   * where `name` is a key, indicating the provider's name; <Br>
   * where `doh_url` is the url of the dns over http; <br>
   * where `vote_weight` is its weight power of influencing the answer. <br> <br>
    
2. Run `cargo run --package api --bin server`. This will start the api.<br>
By default, the api's address is `127.0.0.1:3030`

### interacting with the api
Open the `postman` app or anything similar.

* To add a provider: `POST localhost:3030/provider` <br>
sample body in json:
  ```
  {
    "name": "quad9",
    "doh_url": "https://dns.quad9.net:5053/dns-query",
    "vote_weight": 40 
  }
  ```
* To get the providers list: `GET localhost:3030/providers`
* To resolve: `GET localhost:3030/resolve?name=<hostname>`