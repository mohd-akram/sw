sw
==

`sw` is a stopwatch program for the terminal.

Get started
-----------

`cargo install sw`

Usage
-----

    $ sw; sleep 0.5; sw
    Time elapsed: 0.516624176s

    $ sw start
    $ sw elapsed
    1.441450445
    $ sw stop

    $ sw start
    $ sw record
    $ sw record Done
    $ sw elapsed 1
    1.562742693
    $ sw elapsed Done
    4.153111552
    $ sw times
    1    1.562742693 1.562742693
    Done 4.153111552 2.590368859
    $ sw stop

    $ sw start
    $ git clone -q https://github.com/mohd-akram/sw.git
    $ printf 'Cloning took %.3f seconds\n' `sw -1 lap` >&2
    Cloning took 1.815 seconds
    $ cd sw
    $ cargo build -q
    $ printf 'Building took %.3f seconds\n' `sw -1 lap` >&2
    Building took 1.016 seconds
    $ sw stop
