#!/usr/bin/env bash

tail -n +1 main.asm hello.asm && printf "\n" && \
  rasm assemble -b 1024 main.asm && \
  rasm assemble -b 1024  hello.asm && \
  rlink main.pobj hello.pobj --format macro --standalone --debug -o test.bfm && \
  bfm expand test.bfm -o out.bf && printf "\n\nExecuting compiled vm:\n\n" &&\
  bf out.bf