#include "chibicc.h"

int main(int argc, char** argv) {
  if (argc != 2) {
    error("%s: invalid number of arguments\n", argv[0]);
  }

  Token* tok = tokenize(argv[1]);
  show_tokens(tok);

  Node* node = parse(tok);
  for (Node* it = node; it != NULL; it = it->next) {
    show_trees(it, NULL, false);
  }

  codegen(node);

  return 0;
}
