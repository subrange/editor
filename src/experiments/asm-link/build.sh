#!/usr/bin/env bash

tail -n +1 main.asm func.asm && echo "\n" && \
  rasm assemble main.asm && \
  rasm assemble func.asm && \
  rlink main.pobj func.pobj --format macro -o test.bfm && \
  cat test.bfm