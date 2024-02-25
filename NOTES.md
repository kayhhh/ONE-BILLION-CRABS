# First attempt

https://github.com/kayhhh/ONE-BILLION-CRABS/blob/f428e841bc2e6bfd9f58368fbedd8a73ffc624b8/src/lib.rs

I created about as simple a solution as I could:

- Read file to string
- Iterate over each line
- Use a HashMap to store a running count, max, mean, min for each item
- Collect map to a vector
- Sort the vector
- Iterate over the vector and write to out file

I should note that the output isn't validated, I'm just assuming it's correct xd.

## Result

```
real	1m37.583s
user	1m28.185s
sys	0m9.320s
```
