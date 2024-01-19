# site-to-rss
## use simple filters on your favorite sites to generate xml documents for your personal rss feeds
[try it here!](https://site2rss.protolemon.com/)

This is a project that takes information from the user on how to parse raw html content
into an xml document that's compatible with rss feeds.

It was done using zero javascript just for the love of the challenge and... my hate of javascript. This means
everything is rendered on the backend through [rocket rs](https://rocket.rs/) which I mostly used because
it seemed blazingly fast and interesting, so I decided to learn something new.

(Mostly) everything is deployed on aws via terraform.

## what now?
There are a lot of incomplete things. Namely, backend rendering with easy usability is tough. There are
definitely better frameworks for template rendering and certainly some spots that might call for client-side
javascript.

Other interesting directions would be to implement some smart caching behavior. Users are unlikely to need
their rss feed updated as soon as the site is changed so it would be smart to cache the xml document for a
couple hours. This would also make it load MUCH faster as it could be cached on the edge (closer to them) and wouldn't
have to be generated in real time.

### Design Ideas
these are just some of the notes I took during the beginning of the project. It's mostly word vomit but I'll leave it
here for now.


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
