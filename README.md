# PGGC-Core
gRPC

## debug
create `.env` file and set `VERIFY_TOKEN=hogehoge`
```
cargo run --bin server
```

### client debugging

for macuser, please install [grpcurl](https://github.com/fullstorydev/grpcurl)
```
brew install grpcurl
```

and testing here.
```
grpcurl -plaintext -import-path ./proto -proto judge.proto \
-d '{"team": "alfa", "problem_id": "1", "problem_name": "tutorial"}' \
-rpc-header 'authorization: Bearer token' \
'127.0.0.1:50051' judgement.Judger/Judge;
```
