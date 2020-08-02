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
```

## Optional Variables/Includes (2.0.0)
You can make a variable optional or required by appending a '?' or '!' to the
end of the name, respectively. For example, if you want to require that the
variable `foo` *must always* be replaced with a value, you can write `$foo!`. A
program conforming to this spec should return an error if a document is
finalized with a required variable that is not filled in. Variables default to
*optional*.

In a similar sense, an include can be made optional like this:
`@include? foo.txt`. Includes default to *required*.
