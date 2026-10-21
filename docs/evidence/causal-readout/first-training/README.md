# First registered causal readout training

Source `d1c2a40ac8534ad7893770b3b93a752d94af7277`. All three registered training lives finished. No new test rows were decoded. The unimodal candidate lost its own complete-task comparisons in MNIST (64 vs 76 reward), HAR (61 vs 65) and Fashion (55 vs 59). No candidate was admitted. This attempt failed a necessary condition and is not a useful improvement.

The complete traces, live/frozen states, registered protocol and original executable are retained. Decompress `lives/frozen-aeterna-v1.gz` into a separate writable directory and use that executable with the scripts from the source commit for exact native replay.
