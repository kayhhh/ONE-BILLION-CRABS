# I. First attempt

https://github.com/kayhhh/ONE-BILLION-CRABS/blob/f428e841bc2e6bfd9f58368fbedd8a73ffc624b8/src/lib.rs

I created about as simple a solution as I could:

- Read file to string
- Iterate over each line
- Use a HashMap to store a running count, max, mean, min for each item
- Collect map to a vector
- Sort the vector
- Iterate over the vector and write to out file

I ended up changing mean to a total, then calculating the mean at the end, but it didn't seem to make a difference.
I should also note that the output isn't validated, I'm just assuming it's correct xd.

## Result

```
real	1m37.583s
user	1m28.185s
sys	0m9.320s
```

# II. Tokio

After trying a few times to add multithreading using stdlib, I eventually gave up and added Tokio.
The main improvement here is a chunk-based reading of the file.

Rather than iterating syncronously through each line in the file, we read 16MB chunks and process them in parallel.
Before sending off the chunks to be processed, we need to ensure that we don't split a line in half, so we split our chunk
at the last end line character in the chunk and add the remaining bytes to the next chunk.

The chunk size has a big impact on performance.
A 512KB chunk size, for example, was about the same speed as the syncronous version.
After trying a few different sizes, 16MB seemed to work best for my machine.

There are a lot of unnecessary allocations and copies, but even so this parallel line reading brought a 10x increase in speed.

## Result

```
real	0m13.457s
user	6m38.075s
sys	0m3.690s
```
