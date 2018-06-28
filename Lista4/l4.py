from pylatex import Document, Section, Subsection, Command, Package, MiniPage, Center
from pylatex.utils import NoEscape, bold

def gera_titulo(titulo, subtitulo = None):
    return NoEscape(r'\textbf{'+titulo+r'}' + ((r'\\\Large '+subtitulo) if subtitulo else r''))

class DefineColor(Command):
    def __init__(self, arguments, options = None):
        super().__init__('definecolor', arguments, options)

class Exercicio(Document):
    style_options = [
        "escapeinside=||",
        "bgcolor=darkergray",
        "style=pseudo",
        "breaklines",
        "encoding=utf8",
        "frame=lines",
        "numbersep=2pt",
        # "linenos"
    ]

    def __init__(self, lista, exercicio):
        super().__init__()
        self.ex = exercicio
        self.titulo = NoEscape('Lista ' + str(lista) + r' - ' + 'Exercício ' + str(exercicio))

        self.packages.append(Package('babel', 'brazilian'))
        self.packages.append(Package('fontenc', 'T1'))
        self.packages.append(Package('geometry', ['a4paper', 'margin=2cm']))
        self.packages.append(Package('amsmath'))
        self.packages.append(Package('amssymb'))

        self.packages.append(Package('inputenc', 'utf8'))
        self.packages.append(Package('textgreek'))
        self.preamble.append(Command('DeclareUnicodeCharacter', ['3BB', Command('textlambda')]))
        self.preamble.append(Command('DeclareUnicodeCharacter', ['2260', Command('neq')]))

        self.packages.append(Package('titlesec'))
        self.preamble.append(Command('titleformat', [Command('section'), NoEscape(r'\filcenter\Large'), '', '1em', '']))
        self.preamble.append(Command('titleformat', Command('subsection'), 'runin', extra_arguments=[NoEscape(r'\normalfont\large\bfseries'), Command('thesubsection'), '1em', '']))

        self.packages.append(Package('minted', 'cache=false'))
        self.packages.append(Package('xcolor'))
        self.packages.append(DefineColor(['darkergray', 'rgb', '.86,.86,.86']))

        self.preamble.append(Command('title', gera_titulo('Estrutura de Dados - MC202 A', '1º Semestre de 2018')))
        self.preamble.append(Command('author', bold('Tiago de Paula Alves - 187679')))
        self.preamble.append(Command('date', ''))
        self.append(Command('maketitle'))

    def adiciona_codigo(self, arq, width=1.0):
        with self.create(Center()):
            with self.create(MiniPage(width=str(width)+r"\textwidth")):
                self.append(Command('inputminted', ['pseudo', arq], self.style_options))

    def apende(self, obj):
        self.append(NoEscape(obj))

    def comeca(self):
        return self.create(Section(self.titulo, False))

    def inserir_tex(self, arq):
        self.append(Command('input', arq))

    def ler_ex(self):
        self.inserir_tex('ex' + str(self.ex) + '.tex')

if __name__ == '__main__':
    doc = Exercicio(4, 2)
    with doc.comeca():
        doc.ler_ex()
        doc.adiciona_codigo('ex2.pseudo')
    doc.generate_pdf('l4ex2', compiler_args=['-shell-escape'])

    doc = Exercicio(4, 3)
    with doc.comeca():
        doc.ler_ex()
        doc.adiciona_codigo('ex3.pseudo')
    doc.generate_pdf('l4ex3', compiler_args=['-shell-escape'])
