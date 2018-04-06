#ifndef _PARTICIPANTE_H
#define _PARTICIPANTE_H

#include <stdlib.h>

typedef struct _part *Participante;

Participante constroi_part(unsigned id, unsigned habilidade);
void destroi_part(Participante part);

Participante partida(Participante part1, Participante part2, unsigned regeneracao);
unsigned pegar_id(Participante part);

#endif