# Bempline
## The Bempline Language
The Bempline language is meant to be simple and straightforward. I don't think
if's and else's or for's are needed in a template engine, but I'm not wearing
my glasses.  

I just want something simple and easy that allows me to include a file in the
middle of another, and replace variables.

All Bempline code is between a curly brace and tilde, {~ like this ~}. There must
be a space between the tilde and the Bempline code. The first character must be
a `$` or `@` followed immediately by a non-numeric, non-whitespace character.

#### Including Other Files
```
{~ @include copyright.txt ~}

Chapter One:
```

#### Filling In Variables
```
Dear Customer,

The product you requested is {~ $productPrice ~}. Shipping is an additional
{~ $shippingRate ~}.

Sincerely,
Coorp.
```

#### Patterns
```
{~ @pattern post ~}
<div class="post">
	<h1>{~ $title ~}</h1>
	<p>{~ $summary ~}</p>
</div>
{~ @end-pattern ~}
