# Contributing

Vitrunda follows a small-core engineering rule: keep the resident path cheap and add visual complexity only after the previous layer works end to end.

Before proposing a change, check that it does not introduce an idle polling loop, Chromium/WebView dependency, local web server, or unnecessary background worker.
