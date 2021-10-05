# How to 

### Run the tests for a given step

```
❯ make test^rust^step3
```

## Test against a given command

```
make step3_env && echo "(+ 1 1)" | ./step3_env
```

## Run a step against previous tests:

```
❯ env STEP=step3_env MAL_IMPL=js ./runtest.py  --deferrable --optional tests/step2_eval.mal -- impls/rust/run
           ^binary                                                     ^tests to run
```