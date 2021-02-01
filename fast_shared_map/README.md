# Fast Shared Map
Searching fo a fast shared MAP updated in real-time across multiple servers.

Candidates:
* ArcSwap (t_arcswap) 
* Arc<Map> shared using channels (t_bottle). A BTreeMap is encapsulated in a Arc<> and then sent over a channel.
* RWLock (t_rwlock)

## Results

Read only withouth updates. 3 readers, 1.000.000 operations reads each thread
```
t_arcswap t_bottle t_rwlock (Less is better in millisecs)
63437 43182 393694
70922 49161 385529
62314 43477 357077
67356 43505 408065
66283 48010 388920

Avg 3027 4398 517 Processed/millisecond (More is better in millisecs)

```

Read and update . 3 readers, 1 writer, 500 updates, 1.000.000 reads each thread. Datasets of 1.000 elements
```
READ WRITE
t_arcswap t_bottle t_rwlock (Less is better in millisecs)
50807 49838 406025
50291 54390 429696
71849 72609 407370
64165 47694 417333
48655 46515 399049

Avg 3499 3689 485 Processed/millisecond (More is better in millisecs)
```

Read and update . 3 readers, 1 writer, 500 updates, 1.000.000 reads each thread. Datasets of 10.000 elements
```
READ WRITE BIG (Less is better)
t_arcswap t_bottle t_rwlock
61362 49649 367206
62827 49549 444980
63316 50748 457833
62003 51198 460151
62223 51459 506778
Avg 3207 3958 447 Processed/millisecond (More is better in millisecs)
```

Read and update . 3 readers, 1 writer, 100 updates, 1.000.000 reads each thread. Datasets of 100.000 elements
```
READ WRITE GIGANT (Less is better)
t_arcswap t_bottle t_rwlock
69940 50830 454824
71035 50766 448154
77162 52339 491358
71893 52727 457284
73849 52789 467531
Avg 2748 3854 431 Processed/millisecond (More is better in millisecs)
```


## Conclusion
It is faster to transmit a message than to access the same resource to write and read. The downside is that it involves a complete update of the dataset, because once inside an Arc, it cannot be modified. 