#!/bin/bash
flatc --rust -o src/fbs/ fbs/*.fbs
flatc --ts -o ts/fbs/ fbs/*.fbs
