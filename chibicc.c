#include "chibicc.h"

int main(int argc, char** argv) {
  if (argc != 2) {
    error("%s: invalid number of arguments\n", argv[0]);
  }

  Token* tok = tokenize(argv[1]);
  // show_tokens(tok);

  Function* prog = parse(tok);
  for (Node* it = prog->body; it != NULL; it = it->next) {
    // show_trees(it, NULL, false);
  }

  codegen(prog);

  return 0;
}
