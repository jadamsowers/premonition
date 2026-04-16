cmake_minimum_required(VERSION 3.20)
project(Premonition VERSION 0.1.0)

# 1. Provide path to JUCE (via FetchContent or local installation)
include(FetchContent)
FetchContent_Declare(
    JUCE
    GIT_REPOSITORY https://github.com/juce-framework/JUCE.git
    GIT_TAG        7.0.5
    GIT_SHALLOW    TRUE
    GIT_PROGRESS   TRUE
)
set(FETCHCONTENT_QUIET FALSE)
FetchContent_MakeAvailable(JUCE)

juce_add_plugin(Premonition
    VERSION "0.1.0"
    PLUGIN_MANUFACTURER_CODE JADM
    PLUGIN_CODE Prem
    FORMATS VST3 AU Standalone
    PRODUCT_NAME "Premonition"
)

# 2. Setup Cargo Build via custom command
set(CORE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../premonition-core)
set(RUST_LIB ${CMAKE_CURRENT_SOURCE_DIR}/../target/release/libpremonition_core.a)

add_custom_command(
    OUTPUT ${RUST_LIB}
    COMMAND cargo build --release --manifest-path ${CORE_DIR}/Cargo.toml
    DEPENDS ${CORE_DIR}/src/lib.rs
    COMMENT "Building premonition-core via Cargo"
)
add_custom_target(PremonitionCoreTarget DEPENDS ${RUST_LIB})

# 3. Add Sources
target_sources(Premonition PRIVATE
    Source/PluginProcessor.cpp
    Source/PluginProcessor.h
    Source/PremonitionWrapper.h
)

# 4. Link JUCE and Rust DSP Core
target_link_libraries(Premonition
    PRIVATE
        juce::juce_audio_utils
        juce::juce_audio_processors
        juce::juce_audio_basics
    PUBLIC
        ${RUST_LIB}
)
target_include_directories(Premonition PRIVATE Source)
add_dependencies(Premonition PremonitionCoreTarget)
