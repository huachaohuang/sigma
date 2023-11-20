# The Sigma Programming Language

Sigma is a scripting language for data processing. It combines the power of relational calculus and modern programming languages.

## Usage

```
cargo run
```

It runs an interactive shell like this:

```
Sigma 0.0.1
>>>
```

Then input some statements documented below and see what happens! 

Sigma has a built-in json module. You can use Sigma as a command-line tool to process json data:

```
>>> import json
>>> data = json.load("/path/to/file.json")
[{name: "richard", email: "richard@example.com"}]
>>> from x in data where x.name == "richard" select x.email
["richard@example.com"]
```

## Statements

Syntax:

```
Statement = ImportStatement | ExpressionStatement
```

### Import statement

Syntax:

```
ImportStatement = 'import' NAME
```

Example:

```
import json
```

### Expression statement

Syntax:

```
ExpressionStatement = Expression
```

## Expressions

Syntax:

```
Expression = LiteralExpression
           | ListExpression
           | HashExpression
           | IndexExpression
           | FieldExpression
           | OperatorExpression
           | AssignmentExpression
           | CollectionExpression
```

### Literal expression

Syntax:

```
LiteralExpression = str | i64 | f64 | bool
```

String literal:

```
>>> "abc"
>>> "abc\"123\""
```

Number literal:

```
>>> 123
>>> 123.456
```

Boolean literal:

```
>>> true
>>> false
```

### List and index expression

Syntax:

```
ListExpression = '[' (Expression ',')* Expression? ']'

IndexExpression = Expression '[' Expression ']'
```

```
>>> list = [1, 2, 3]
>>> list[1]
2
```

### Hash and field expression

Syntax:

```
PairExpression = Expression ':' Expression
HashExpression = '{' (PairExpression ',')* PairExpression? '}'

FieldExpression = Expression '.' NAME
```

```
>>> hash = {a: 1, b: "2"}
>>> hash.a
1
>>> hash["a"]
1
```

### Operator expression

Syntax:

```
OperatorExpression = ArithmeticExpression | ComparisonExpression | LazyBooleanExpression
```

#### Arithmetic operator

Syntax:

```
UnaryOperator = '-' | '!'
UnaryExpression = UnaryOperator Expression

BinaryOperator = '+' | '-' | '*' | '/' | '%'
               | '|' | '^' | '&' | '<<' | '>>'
BinaryExpression = Expression BinaryOperator Expression
```

Example:

```
>>> -1
-1
>>> !true
false
>>> 1 + 2 - 3 + 4
4
>>> (1 + 2) * 3 / 4
2
>>> (1 | 2) & 3 | 4
7
```

#### Comparison operator

Syntax:

```
ComparisonOperator = '==' | '!=' | '<' | '<=' | '>' | '>=' | 'in' | 'not' 'in'
ComparisonExpression = Expression ComparisonOperator Expression
```

Example:

```
>>> 1 < 2
true
>>> list = [1, 2, 3]
>>> 1 in list
true
>>> 1 not in list
false
```

#### LazyBoolean operator


Syntax:

```
LazyBooleanOperator = '||' | '&&'
LazyBooleanExpression = Expression LazyBooleanOperator Expression
```

Example:

```
>>> 1 < 2 || 2 < 1
true
>>> 1 < 2 && 2 < 1
false
```

### Assignment expression

Syntax:

```
AssignmentExpression = Expression '=' Expression

CompoundAssignmentOperator = '+=' | '-=' | '*=' | '/=' | '%=' 
                           | '|=' | '^=' | '&=' | '<<=' | '>>='
CompoundAssignmentExpression = Expression CompoundAssignmentOperator Expression
```

Example:

```
>>> a = 1 + 2 * 3
7
>>> a *= 2
14
>>> a |= 1
15
```

### Collection expression

Syntax:

```
CollectionExpression = InsertExpression
                     | UpdateExpression
                     | DeleteExpression
                     | SelectExpression

FromClause = 'from' NAME 'in' Expression
JoinClause = 'join' NAME 'in' Expression ('on' Expression)?
WhereClause = 'where' Expression
```

#### Insert expression

Syntax:

```
InsertExpression = 'into' Expression 'insert' Expression (',' Expression)*
```

Example:

```
>>> list = [1, 2]
>>> into list insert 3, 4
2
>>> list
[1, 2, 3, 4]
```

#### Update expression

Syntax:

```
UpdateExpression = FromClause WhereClause? 'update' Expression (',' Expression)*
```

Example:

```
>>> list = [1, 2, 3]
>>> from x in list update x += 1
3
>>> list
[2, 3, 4]
>>> list = from x in [1, 2, 3] select {a: x, b: x}
[{a: 1, b: 1}, {a: 2, b: 2}, {a: 3, b: 3}]
>>> from x in list where x.a > 1 update x.a += 1, x.b *= 2
2
>>> list
[{a: 1, b: 1}, {a: 3, b: 4}, {a: 4, b: 6}]
```

#### Delete expression

Syntax:

```
DeleteExpression = FromClause WhereClause? 'delete' NAME (',' NAME)*
```

Example:

```
>>> list = [1, 2, 3]
>>> from x in list where x % 2 == 0 delete x
1
>>> list
[1, 3]
>>> from x in list delete x
2
>>> list
[]
```

#### Select expression

Syntax:

```
SelectExpression = FromClause JoinClause? WhereClause? ('select' Expression)?
```

```
>>> hash1 = from x in [1, 2, 3] select {a: x, b: x * 10}
[{a: 1, b: 10}, {a: 2, b: 20}, {a: 3, b: 30}]
>>> hash2 = from x in [2, 3, 4] select {a: x, b: x * 100}
[{a: 2, b: 200}, {a: 3, b: 300}, {a: 4, b: 400}]
>>> from x1 in hash1 join x2 in hash2 on x1.a == x2.a
[{x1: {a: 2, b: 20}, x2: {a: 2, b: 200}}, {x1: {a: 3, b: 30}, x2: {a: 3, b: 300}}]
>>> from x1 in hash1 join x2 in hash2 on x1.a == x2.a where x1.a > 2 select {b1: x1.b, b2: x2.b}
[{b1: 30, b2: 300}]
```