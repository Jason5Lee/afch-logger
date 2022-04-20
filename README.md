# afch-logger

Logger for Azure Function Custom Handler,
abusing the undocumented (at least I don't know where) rule of e Function
"infering" the log level from stderr.

## Usage

You can initialize the log by `afch_logger::init_logger`. If you want to use the replacement logic
in other logger, you can call `afch_logger::to_error_log` to get the replaced error log, and `afch_logger::contains_warn`
to test whether the message contains `warn` (case insensitive).

## Strategy

For Azure Function Custom Handler, if you print a message to stdout, it will be considered as a `Information` 
level log by Azure Function runtime.

If you print a message to stderr, then it will be consider `Error` if it does not contain `warn` (case insensitive),
otherwise it will be `Warning`.

So the strategy is, for error-level log, we find the occurence of `warn` and replace `r` by `𝗋`(\U+1d5cb)
and `R` by `𝖱`(\U+1d5b1). For warning-level log, if `warn` does not occur, we add a `warning:` prefix.

