// Premonition JUCE Plugin Processor Implementation

#include "PluginProcessor.h"

juce::AudioProcessorValueTreeState::ParameterLayout PremonitionProcessor::createParameterLayout()
{
    juce::AudioProcessorValueTreeState::ParameterLayout layout;
    
    // Minimal subset of parameters for testing connection
    layout.add(std::make_unique<juce::AudioParameterFloat>(
        juce::ParameterID{"35", 1}, "Volume", 0.0f, 1.0f, 0.75f));
    layout.add(std::make_unique<juce::AudioParameterFloat>(
        juce::ParameterID{"11", 1}, "Cutoff", 0.0f, 1.0f, 0.25f));
    layout.add(std::make_unique<juce::AudioParameterFloat>(
        juce::ParameterID{"12", 1}, "Resonance", 0.0f, 1.0f, 0.0f));

    return layout;
}

PremonitionProcessor::PremonitionProcessor()
    : apvts(*this, nullptr, "Parameters", createParameterLayout())
{
    engine = premonition_create();
}

PremonitionProcessor::~PremonitionProcessor()
{
    if (engine) {
        premonition_destroy(engine);
        engine = nullptr;
    }
}

void PremonitionProcessor::prepareToPlay(double sampleRate, int samplesPerBlock)
{
    currentSampleRate = sampleRate;
    currentBlockSize = samplesPerBlock;
    
    if (engine) {
        premonition_init(engine, (float)sampleRate);
    }
}

void PremonitionProcessor::releaseResources()
{
}

bool PremonitionProcessor::isBusesLayoutSupported(const BusesLayout& layouts) const
{
    if (layouts.getMainOutputChannelSet() != juce::AudioChannelSet::stereo())
        return false;
    
    if (layouts.getMainInputChannelSet() != juce::AudioChannelSet::stereo() &&
        !layouts.getMainInputChannelSet().isDisabled())
        return false;
    
    return true;
}

void PremonitionProcessor::processBlock(juce::AudioBuffer<float>& buffer, 
                                        juce::MidiBuffer& midiMessages)
{
    // Update parameters
    if (engine) {
        premonition_set_param(engine, 35, *apvts.getRawParameterValue("35"));
        premonition_set_param(engine, 11, *apvts.getRawParameterValue("11"));
        premonition_set_param(engine, 12, *apvts.getRawParameterValue("12"));
    }

    auto* left = buffer.getWritePointer(0);
    auto* right = buffer.getWritePointer(1);
    int numSamples = buffer.getNumSamples();
    
    // Process MIDI messages
    for (const auto metadata : midiMessages) {
        auto message = metadata.getMessage();
        
        if (message.isNoteOn()) {
            if (engine) premonition_note_on(engine, message.getChannel(), message.getNoteNumber(), message.getVelocity());
        }
        else if (message.isNoteOff()) {
            if (engine) premonition_note_off(engine, message.getChannel(), message.getNoteNumber());
        }
    }
    
    // Process audio
    if (engine) {
        premonition_process(engine, left, right, numSamples, (float)currentSampleRate);
    } else {
        buffer.clear();
    }
}

juce::AudioProcessorEditor* PremonitionProcessor::createEditor()
{
    return new juce::GenericAudioProcessorEditor(*this);
}

bool PremonitionProcessor::hasEditor() const
{
    return true;
}

const juce::String PremonitionProcessor::getName() const
{
    return "Premonition";
}

bool PremonitionProcessor::acceptsMidi() const
{
    return true;
}

bool PremonitionProcessor::producesMidi() const
{
    return false;
}

bool PremonitionProcessor::isMidiEffect() const
{
    return false;
}

double PremonitionProcessor::getTailLengthSeconds() const
{
    return 0.0;
}

int PremonitionProcessor::getNumPrograms()
{
    return 1;
}

int PremonitionProcessor::getCurrentProgram()
{
    return 0;
}

void PremonitionProcessor::setCurrentProgram(int index)
{
}

const juce::String PremonitionProcessor::getProgramName(int index)
{
    return "Init";
}

void PremonitionProcessor::changeProgramName(int index, const juce::String& newName)
{
}

void PremonitionProcessor::getStateInformation(juce::MemoryBlock& destData)
{
    // TODO: Export parameters as JSON
}

void PremonitionProcessor::setStateInformation(const void* data, int sizeInBytes)
{
    // TODO: Import parameters from JSON
}
