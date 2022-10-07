#!/bin/bash
flatc --rust -o src/ src/fbs/*.fbs
flatc --ts -o src/ts/ src/fbs/*.fbs
