Loglux
======

[![License](https://img.shields.io/badge/license-UNLICENSE-blue.svg?style=flat)](https://github.com/rarescosma/loglux/blob/master/UNLICENSE)

Loglux is a simple rust application with portable binaries to control brightness.

While heavily inspired by the original [lux][lux], it differs from it in one major aspect:

The brightness control is [logarithmic][weber-fechner] - as we approach darker and
darker brightness values the control step gets smaller and smaller.

It's perfect for us creatures of the night who get to sleep watching rust streams in complete
darkness at 1AM as it allows us to make the laptop screen *really* dark.

## Installation

Binaries for Linux on various architectures are available on the [releases][releases] page.

They are statically linked against [musl][musl] to completely reduce runtime dependencies.

## Usage

```
loglux OPERATION [-p|--path (default: /sys/class/backlight)] [-n|--num-steps (default: 75)]
```

* `OPERATION` is either `up` or `down`
* `--path` can be either a start directory containing multiple controllers, or a path to specific controller.
  In the directory case, the controller with the highest `max_brightness` setting will be selected.
* `--num-steps` is the only tunable parameter and it specifies the total number of steps for the
  adjustment scale. The default is tuned for steps of 9-10% near the maximum, then they'll get smaller
  and smaller as we approach the minimum.

[lux]: https://github.com/Ventto/lux

[weber-fechner]: https://en.wikipedia.org/wiki/Weber%E2%80%93Fechner_law

[releases]: https://github.com/rarescosma/loglux/releases

[musl]: https://musl.libc.org/
