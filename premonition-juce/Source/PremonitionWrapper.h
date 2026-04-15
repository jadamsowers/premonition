#pragma once

#include <stdint.h>
#include <stddef.h>

extern "C" {

// Opaque struct representing the Engine
struct PremonitionEngine;

// Lifecycle
PremonitionEngine* premonition_create();
void premonition_destroy(PremonitionEngine* engine);
void premonition_init(PremonitionEngine* engine, float sample_rate);

// Audio Processing
void premonition_process(
    PremonitionEngine* engine,
    float* output_l,
    float* output_r,
    size_t num_samples,
    float sample_rate
);

// MIDI
void premonition_note_on(PremonitionEngine* engine, uint8_t channel, uint8_t note, uint8_t velocity);
void premonition_note_off(PremonitionEngine* engine, uint8_t channel, uint8_t note);

// Parameters
void premonition_set_param(PremonitionEngine* engine, uint32_t param_id, float value);
float premonition_get_param(PremonitionEngine* engine, uint32_t param_id);

void premonition_panic();

} // extern "C"
