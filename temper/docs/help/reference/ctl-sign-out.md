# temper ctl sign-out

Kind: reference

End the sign-in and leave the token empty.

## sign-out

```
sign-out
End the sign-in and leave the token empty
Prints: Nothing
--json: the record sign_out.
Key: none.
It sends none of the requests DIR keeps. If the engine answers
and refuses the token, it still empties token, exits 0 and prints
nothing: the sign-in is over either way. With no engine answer,
it keeps token and exits 1, naming the address that did not answer.
Use sign-out before a link's sign-in into an already signed-in DIR.
```
