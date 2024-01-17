# site-to-rss

### Design Ideas
rocket rs /generate-xml:
- take forum post request -> save config to db with hash as key

rocket rs /\[hash-key\].xml function:
- pull config from db using hash-key
- download html from link in config
- apply regex rules to get iterable
- generate handlebars template using iterable
- this function should be cached w/ timeout^

attempt to parse regex using FastGPT?

how to convert user regex to real regex?

some sort of preview function to show current regex selection?
this can be done with html if it opens in a new page.


