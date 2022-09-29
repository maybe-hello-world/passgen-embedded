#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct GenerationResult {
  bool success;
  const uint8_t *error_string;
} GenerationResult;

struct GenerationResult generate_password(uint8_t *buffer, uintptr_t length, uint64_t random_state);
