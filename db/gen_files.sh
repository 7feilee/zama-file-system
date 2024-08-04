#!/bin/bash

mkdir -p db/uploads

for i in $(seq 1 100); do
  filename="db/uploads/file_$i.txt"
  
  head -c 1024 </dev/urandom >"$filename"
  
  echo "Generated $filename"
done

echo "100 random files generated in the 'uploads' folder."