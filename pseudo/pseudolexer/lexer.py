from pygments.lexer import RegexLexer
from pygments import token

declaration = 'Algoritmo'

types = [
    'Entrada(s)?:',
    'Saída(s)?:',
    'verdadeiro',
    'falso', 'não',
    '∞'
]

keywords = [
    'adjacente a',
    'não contém',
    'contém',
    'para cada',
    'tal que',
    r'(?<=para cada \w )em(?= \w)',
    'para',
    r'(?<=para \w )de(?= \w)',
    'até', 'já',
    'senão', 'se',
    'enquanto',
    'faça',
    ' e ', ' é ',
    ' não foi ',
    'encerra ',
    'lança ',
]

comment_token = token.Token.TextComment
token.STANDARD_TYPES[comment_token] = 'ct'

class PseudoLexer(RegexLexer):
    name = 'Pseudo'
    aliases = ['pseudo']
    filenames = ['*.pseudo']

    tokens = {
        'root': [
            (r'(?<=lança )\w+', token.Name.Exception),
            (r'máx(_k)?|novo_l', token.Name.Variable),
            (declaration, token.Keyword.Declaration),
            (r'(?<='+declaration+r' )\w*', token.Name.Function),
            (r'|'.join(types), token.Keyword.Type),
            (r'|'.join(keywords), token.Keyword),
            (r'←|\+| \- |\.|λ|\/|mod|\>|\<|\*|=|≠', token.Operator),
            (r'::|-->|\.{3}', token.Operator.Word),
            (r'\d+|\d*,\d+', token.Number),
            (r':|,|\[|\]|\(|\)|\{|\}|;', token.Punctuation),
            (r'\w\w+', token.Text),
            (r'\w', token.Name.Variable),
            (r'\s+', token.Text),
            (r'--.*?$', comment_token),
        ]
    }
