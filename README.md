# afch-logger

Logger for Azure Function Custom Handler,
abusing the undocumented (at least I don't know where) rule of e Function
"infering" the log level from stderr.

## Usage

You can initialize the log by `afch_logger::init`. You can also implement your own transforming
by implementing `afch_logger::Transform` trait and passing it to `afch_logger::init_transform`.

## Strategy

For Azure Function Custom Handler, if you print a message to stdout, it will be considered as a `Information` 
level log by Azure Function runtime.

If you print a message to stderr, then it will be consider `Error` if it does not contain `warn` (case insensitive),
otherwise it will be `Warning`.

So the default strategy is, for error-level log, if `warn` occurs, base64-encode it, if the encoded string still contains `warn`,
base-encode again, and if the twice-encoded string still contains `warn` (which should be impossible), log an error explain that the
following warning is error, then log it as a warning. For warning-level log, if `warn` does not occur, add a `warning:` prefix.
