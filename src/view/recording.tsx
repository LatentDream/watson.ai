import { Group, Button, SegmentedControl, Stack, Modal, TextInput, Text, useComputedColorScheme, NativeSelect  } from '@mantine/core';
import { FilePlusIcon, PauseIcon, StopIcon, ResumeIcon, HeadingIcon, QuestionMarkIcon } from "@radix-ui/react-icons";
import { useEffect, useState } from "react";
import { meetingFmc, recorderFmc, sessionFmc, settingFmc } from '../controller';
import { useAppContext } from '../AppContext';
import { useDisclosure } from '@mantine/hooks';
import { RichTextEditor, Link } from '@mantine/tiptap';
import { useEditor } from '@tiptap/react';
import Highlight from '@tiptap/extension-highlight';
import StarterKit from '@tiptap/starter-kit';
import Underline from '@tiptap/extension-underline';
import Superscript from '@tiptap/extension-superscript';
import SubScript from '@tiptap/extension-subscript';
import Placeholder from '@tiptap/extension-placeholder';
import { NewMeetingNote } from '../bindings/NewMeetingNote';
import { invoke, window as windowTauri } from "@tauri-apps/api"
import { TauriEvent } from "@tauri-apps/api/event"
import { notifications } from '@mantine/notifications';
import { info } from 'tauri-plugin-log-api';
import React from 'react';


let exit_called = false;


export default function Recording() {

    // Handle window closing
    windowTauri.getCurrent().listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async () => {
        info("Window closing ...")
        if (exit_called) {
            return;
        }
        exit_called = true;
        invoke('exit');
    })

    // @ts-ignore 
    const { views, updateViews, notifyChangeInRecordingState } = useAppContext();
    const [recording, setRecording] = useState(false);
    const [pause, setRecordingPause] = useState(false);
    const [language, setLanguage] = useState("English");
    const [opened, { open, close }] = useDisclosure(false);
    const [note, setNote] = useState<NewMeetingNote | null>(null);
    const computedColorScheme = useComputedColorScheme('light', { getInitialValueInEffect: true });
    const [promptName, setPromptName] = useState('Summarize');
    const [promptsNameList, setPromptsNameList] = useState<string[]>(['Summarize', 'Improved Hand Note']);
    const [prompts, _setPrompt] = useState<Map<string, string>>(new Map());
    const [inputDevices, setInputDevices] = useState<string[]>([]);
    const [outputDevices, setOutputDevices] = useState<string[]>([]);
    const [selectedInputDevice, setSelectedInputDevice] = useState<string>();
    const [selectedOutputDevice, setSelectedOutputDevice] = useState<string>();
    const [height, setHeight] = useState(window.innerHeight - 93);

    React.useEffect(() => {
        window.addEventListener("resize", () => {setHeight(window.innerHeight - 93)});
        return () => window.removeEventListener("resize", () => {setHeight(window.innerHeight - 93)});
    });

    async function stopRecording() {
      /* Logic when recording is stop
       * 1. Save all variables for async processing
       * 2. Stop the recording 
       * 3. Add meeting info (if user close the session during transcription, infos won't be loss)
       * 4. Transcribe & Summarize the meeting
       */
      info("stop recording called")
      let lang = language == "English" ? "En" : "Fr";
      let prompt_to_use = prompts.get(promptName);
      let summarizationType = promptName;
      let user_note = "";
      let user_title = "";
      if (note) {
        user_note = note.note; 
        user_title = note.title;
        await sessionFmc.set_new_meeting_note({
          title: "",
          note: ""
        });
        noteEditor?.commands.setContent("");
        setNote({note: "", title: ""});
      }
      let meeting = await recorderFmc.stop();
      info("recording stopped");
      meeting.note = user_note
      if (user_title) {
        meeting.title = user_title;
      }
      if (prompt_to_use) {
        meeting.prompt = prompt_to_use;
      }
      await meetingFmc.update(meeting);
      info("Saved note & Title");
      await meetingFmc.increment_async_ops_count(meeting);
      try {
        info("incremented async ops count");
        setRecording(false);
        setRecordingPause(false);
        notifyChangeInRecordingState();
        notifications.show({
          title: 'Recording stopped',
          message: 'Your recording has been sent for transcription and summarization. This may take a few minutes.',
          autoClose: 4000,
          icon: <QuestionMarkIcon />,
          color: 'gray',
          withBorder: true,
          withCloseButton: true,
        });
        info("Starting transcription")
        meeting = await recorderFmc.transcribe(meeting, lang);
        await meetingFmc.update(meeting);
        info("Finished transcription")
        if (summarizationType == "Improved Hand Note") {
          info("improving note")
          await meetingFmc.improve_note(meeting);
          info("improved note")
        } else {
          info("summarizing with prompt: " + meeting.prompt)
          await meetingFmc.summarize(meeting);
          info("summarized meeting")
          await meetingFmc.decrement_async_ops_count(meeting);
          info("decremented async ops count")
          notifications.show({
            title: 'New meeting ready',
            message: 'Your meeting is ready for review and publication!. You can now view it in the meeting list.',
            autoClose: 4000,
            withBorder: true,
            withCloseButton: true,
          });
        }
      } catch (error) {
        console.error('Error fetching setting:', error);
        await meetingFmc.decrement_async_ops_count(meeting);
      }
      updateViews();
    }

    const noteEditor = useEditor({
      extensions: [
        StarterKit,
        Underline,
        Link,
        Superscript,
        SubScript,
        Highlight,
        Placeholder.configure({ placeholder: 'My personal notes ...' })
      ],
      // Init the content of the editor when a new prop is loaded
      async onCreate(props) {
        const note = await sessionFmc.get_new_meeting_note();
        props.editor?.commands.setContent(note.note)
      },
      // triggered on every change
      onUpdate: async ({ editor }) => {
        let content = editor.getHTML();
        if (note) {
          note.note = content;
        }
      },
      content: note?.note,
    });
  
    useEffect(() => {
      async function fetchData() {
        try {
          const result = await recorderFmc.get_state();
          info(result);
          if (result == "Recording") {
            setRecording(true);
            setRecordingPause(false);
          } else if (result == "Paused") {
            setRecording(true);
            setRecordingPause(true);
          } else {
            setRecording(false);
            setRecordingPause(false);
          }
          const new_meeting_note = await sessionFmc.get_new_meeting_note(); 
          setNote(new_meeting_note);
          noteEditor?.commands.setContent(new_meeting_note.note);     
          const settings = await settingFmc.get();
          let promptsNameList: string[] = ['Summarize', 'Improved Hand Note'];
          if (settings.prompts != null) {
            settings.prompts.forEach(element => {
              prompts.set(element.name, element.prompt);
              promptsNameList.push(element.name);
            });
          }
          setPromptsNameList(promptsNameList); // Signal react to update the view
          const devices = await recorderFmc.get_available_audio_devices();
          setInputDevices(devices.input_devices.map(device => {if (device.is_default) {setSelectedInputDevice(device.name);} return device.name;}));
          setOutputDevices(devices.output_devices.map(device => {if (device.is_default) {setSelectedOutputDevice(device.name);} return device.name;}));
          if (recording) {
            let devices = await recorderFmc.get_recording_device_names();
            setSelectedInputDevice(devices.input_device_name);
            setSelectedOutputDevice(devices.output_device_name);
          }
        } catch (error) {
          // Handle any errors here
          console.error('Error fetching setting:', error);
        }
      }
      fetchData();
    }, []); // Effect will run once when the component mounts.

    useEffect(() => () => {
      async function saveData() {
        if (note) {
          info("save: " + note);
          try {
            await sessionFmc.set_new_meeting_note({
              title: note.title,
              note: note.note
            });
          } catch (error) {
            // Handle any errors here
            console.error('Error saving setting:', error);
          }
      }
      }
      saveData();
    }, [note]); // Effect will run when the component unmount

    return (
      <Stack justify="space-between" h={height}>
        <Stack>
        <Modal opened={opened} onClose={close} title="Instruction for meeting processing" size="sm" >
          <Stack>

          <Group>
            <Text>Select language: </Text>
            <SegmentedControl 
              fullWidth 
              color={computedColorScheme === "light" ? 'var(--mantine-color-borealGreen-4)' : 'var(--mantine-color-borealGreen-5)'} 
              defaultValue={language} 
              onChange={(value) => {
                setLanguage(value)
              }
              } 
              data={['English', 'French']} 
            />
          </Group>

          <Group>
            <NativeSelect 
              label="Type of summarization" 
              value={promptName} 
              onChange={(event) => {setPromptName(event.currentTarget.value)}} 
              data={promptsNameList} 
            />
          </Group>

          <Group justify="center">
            <Button 
            variant="outline" 
            leftSection={<StopIcon/>}
            color="red"
            onClick={async () => {
              // logic when recording is stopped ~~~~~~~~~~~~~~~~~~~
              close();
              stopRecording();              
            }}>Stop Recording</Button>
          </Group>
          </Stack>
        </Modal>

        <Group  justify="space-between">
        <Group>
          <Button variant="outline"
          disabled={recording} 
          onClick={async (event) => {
            event.preventDefault(); 
            let input = selectedInputDevice ? selectedInputDevice : "";
            let output = selectedOutputDevice ? selectedOutputDevice : "";
            let notif = await recorderFmc.start(input, output);
            if (notif) {
              notifications.show({
                title: 'Recording started',
                message: notif,
                autoClose: 4000,
                icon: <QuestionMarkIcon />,
                color: 'gray',
                withBorder: true,
                withCloseButton: true,
              });
            }
            setRecording(!recording);
            notifyChangeInRecordingState();
          }}
          leftSection={<FilePlusIcon />} >
            Record
          </Button>
        </Group>
          
        <Group justify="flex-end">
          <Button variant="light" 
          leftSection={<ResumeIcon />}
          disabled={!recording || !pause} 
          onClick={async (event) => {
            event.preventDefault();
            await recorderFmc.resume();
            setRecordingPause(false);
            }}>
            Resume
          </Button>

          <Button variant="light" 
          leftSection={<PauseIcon />}
          disabled={!recording || pause} 
          onClick={async (event) => {
            event.preventDefault();
            await recorderFmc.pause();
            setRecordingPause(true);
          }}>
            Pause
          </Button>

          <Button variant="outline" 
          leftSection={<StopIcon />}
          color="red"
          disabled={!recording} 
          onClick={() => {
            open();
            }}>
            Stop
          </Button>
        </Group>
        </Group>

        <TextInput
            leftSection={<HeadingIcon/>}
            placeholder="Meeting Title"
            value={note?.title}
            onChange={async (event) => {
              if (note) {
                note.title = event.currentTarget.value;
                updateViews();
              }
            }}
        /> 



        <RichTextEditor editor={noteEditor} styles={{ content: { minHeight: '12rem' }}}>
          <RichTextEditor.Toolbar sticky stickyOffset={60}>
            <RichTextEditor.ControlsGroup>
            <RichTextEditor.Bold />
            <RichTextEditor.Italic />
            <RichTextEditor.Underline />
            <RichTextEditor.Strikethrough />
            <RichTextEditor.ClearFormatting />
            <RichTextEditor.Highlight />
            <RichTextEditor.Code />
            </RichTextEditor.ControlsGroup>

            <RichTextEditor.ControlsGroup>
            <RichTextEditor.H1 />
            <RichTextEditor.H2 />
            <RichTextEditor.H3 />
            <RichTextEditor.H4 />
            </RichTextEditor.ControlsGroup>

            <RichTextEditor.ControlsGroup>
            <RichTextEditor.Blockquote />
            <RichTextEditor.Hr />
            <RichTextEditor.BulletList />
            <RichTextEditor.OrderedList />
            </RichTextEditor.ControlsGroup>

            <RichTextEditor.ControlsGroup>
            <RichTextEditor.Link />
            <RichTextEditor.Unlink />
            </RichTextEditor.ControlsGroup>

          </RichTextEditor.Toolbar>

          <RichTextEditor.Content />
        </RichTextEditor>
        
        </Stack>

        <Group justify="center" grow>
        <NativeSelect 
          description="Microphone to record" 
          disabled={recording || inputDevices.length <= 1}
          value={selectedInputDevice}
          onChange={(event) => setSelectedInputDevice(event.currentTarget.value)}
          data={inputDevices}
        />
        <NativeSelect 
          description="Speaker to record" 
          disabled={recording || outputDevices.length <= 1}
          value={selectedOutputDevice}
          onChange={(event) => setSelectedOutputDevice(event.currentTarget.value)}
          data={outputDevices}
        />
        </Group>
        
      </Stack>

    );
}
