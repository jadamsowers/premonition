// Premonition JUCE Plugin Processor Implementation

#include "PluginProcessor.h"

PremonitionProcessor::PremonitionProcessor()
{
    // TODO: Load premonition-core shared library and create engine
}

PremonitionProcessor::~PremonitionProcessor()
{
    // TODO: Destroy engine and unload library
}

void PremonitionProcessor::prepareToPlay(double sampleRate, int samplesPerBlock)
{
    currentSampleRate = sampleRate;
    currentBlockSize = samplesPerBlock;
    
    // TODO: Call premonition_init()
}

void PremonitionProcessor::releaseResources()
{
    // TODO: Cleanup
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
    auto* left = buffer.getWritePointer(0);
    auto* right = buffer.getWritePointer(1);
    int numSamples = buffer.getNumSamples();
    
    // Process MIDI messages
    for (const auto metadata : midiMessages) {
        auto message = metadata.getMessage();
        
        if (message.isNoteOn()) {
            // TODO: Call premonition_note_on()
        }
        else if (message.isNoteOff()) {
            // TODO: Call premonition_note_off()
        }
    }
    
    // TODO: Call premonition_process()
}

juce::AudioProcessorEditor* PremonitionProcessor::createEditor()
{
    return nullptr; // TODO: Create editor
}

bool PremonitionProcessor::hasEditor() const
{
    return false;
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
