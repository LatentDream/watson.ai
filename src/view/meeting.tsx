import { ActionIcon, Center, Checkbox, NativeSelect, TextInput, useComputedColorScheme } from '@mantine/core';
import { HeadingIcon, TrashIcon, Pencil1Icon, MagicWandIcon, ReaderIcon, UploadIcon, FileTextIcon, QuestionMarkIcon, Cross1Icon, CopyIcon } from '@radix-ui/react-icons';
import { Stack, Group, Autocomplete, Button, Modal, Flex, Text } from '@mantine/core';
import { useState, useEffect, useRef } from 'react';
import { crmFmc, meetingFmc, recorderFmc, settingFmc } from '../controller';
import { Meeting } from '../bindings';
import { useLocation  } from 'react-router-dom';
import { useAppContext } from '../AppContext';
import { useDisclosure } from '@mantine/hooks';
import { RichTextEditor, Link } from '@mantine/tiptap';
import { useEditor } from '@tiptap/react';
import Highlight from '@tiptap/extension-highlight';
import StarterKit from '@tiptap/starter-kit';
import Underline from '@tiptap/extension-underline';
import Superscript from '@tiptap/extension-superscript';
import SubScript from '@tiptap/extension-subscript';
import { invoke, window as windowTauri } from "@tauri-apps/api"
import { TauriEvent } from "@tauri-apps/api/event"
import { ArrowLeftIcon } from "@radix-ui/react-icons";
import { SegmentedControl } from '@mantine/core';
import { notifications } from '@mantine/notifications';


async function updateMeeting(meeting: Meeting) {
    await meetingFmc.update(meeting)
        .then(() => {
            console.log("Meeting saved")
        }).catch((err) => {
            console.log(err)
        });
}

async function search_org(query: string) {
    const orgs = await crmFmc.search_orgs(query).catch((err) => {
        console.log(err)
    });
    return orgs;
}

let exit_called = false;

export default function MeetingView() {

    // @ts-ignore 
    const { views, updateViews } = useAppContext();
    const mountedMeetingId = useRef("-1");
    const meetingId = useLocation()["pathname"].split("/")[1];

    const [orgs, setOrgs] = useState<string[]>([]); 
    const [org, setOrg] = useState<string>("");
    const [orgsNameToId, setOrgsNameToId] = useState(new Map<string, string>());
    const [reloadEditor, setReloadEditor] = useState(false);
    const [reloadCompany, setReloadCompany] = useState(false);
    const [meeting, setMeeting] = useState<Meeting | null>(null);
    const [publishPossible, setPublishPossible] = useState(false);
    const [transcriptEditorView, setTranscriptEditorView] = useState(false);
    const [promptEditorView, setPromptEditorView] = useState(false);
    const [displayDate, setDisplayDate] = useState<string>("");
    const [deleteModalOpened, { open: openDeleteModal, close: close }] = useDisclosure(false);
    const [editorSelector, setEditorSelector] = useState('Summary');
    const [openedTranscriptModal, { open: openTranscriptModal, close: closeTranscriptModal }] = useDisclosure(false);
    const [openedPromptModal, { open: openPromptModal, close: closePromptModal }] = useDisclosure(false);
    const [language, setLanguage] = useState("English");
    const computedColorScheme = useComputedColorScheme('light', { getInitialValueInEffect: true });
    const [checkedPublishWithPersonalNote, setCheckedPublishWithPersonalNote] = useState(false);
    const [userPrompts, _setUserPrompts] = useState<Map<string, string>>(new Map());
    const [userPromptsNameList, setUserPromptsNameList] = useState<string[]>([...userPrompts.keys()]);
    const [selectedUserPrompt, setSelectedUserPrompt] = useState('');
    const [meetingTitle, setMeetingTitle] = useState("");
    const [affinityIntegrationEnabled, setAffinityIntegrationEnabled] = useState(false);
    const [companyName, setCompanyName] = useState("");
    


    // Editor Data ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    const summaryEditor = useEditor({
        extensions: [
          StarterKit,
          Underline,
          Link,
          Superscript,
          SubScript,
          Highlight,
        ],
        // Init the content of the editor when a new prop is loaded
        async onCreate(props) {
            const result = await meetingFmc.get(meetingId);
            props.editor?.commands.setContent(result.summary)
        },
        // triggered on every change
        onUpdate: async ({ editor }) => {
          let content = editor.getHTML();
          if (meeting) {
            meeting.summary = content;
          }
        },
        content: meeting?.summary,
    });

    const noteEditor = useEditor({
        extensions: [
          StarterKit,
          Underline,
          Link,
          Superscript,
          SubScript,
          Highlight,
        ],
        // Init the content of the editor when a new prop is loaded
        async onCreate(props) {
            const result = await meetingFmc.get(meetingId);
            props.editor?.commands.setContent(result.note)
        },
        // triggered on every change
        onUpdate: async ({ editor }) => {
          let content = editor.getHTML();
          if (meeting) {
            meeting.note = content;
          }
        },
        content: meeting?.note,
    });
    
    const transcriptEditor = useEditor({
        extensions: [
            StarterKit,
        ],
        // Init the content of the editor when a new prop is loaded
        async onCreate(props) {
            const result = await meetingFmc.get(meetingId);
            props.editor?.commands.setContent(result.transcript)
        },
        // triggered on every change
        onUpdate: async ({ editor }) => {
            let content = editor.getText();
            if (meeting) {
              meeting.transcript = content;
            }
          },
        content: meeting?.transcript,
        
    });

    const promptEditor = useEditor({
        extensions: [
            StarterKit,
        ],
        // Init the content of the editor when a new prop is loaded
        async onCreate(props) {
            const result = await meetingFmc.get(meetingId);
            props.editor?.commands.setContent(result.prompt)
        },
        // triggered on every change
        onUpdate: async ({ editor }) => {
            let content = editor.getText();
            if (meeting) {
              meeting.prompt = content;
            }
          },
        content: meeting?.prompt,
        
    });    
    
    // Component effect ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    useEffect(() => {
        // Load meeting data -> Called when meetingId updates or mounts
        setTranscriptEditorView(false);
        setPromptEditorView(false);
        console.log("MeetingId changed", meetingId);
        async function fetchData() {
          try {
            // Reload the prop - start
            mountedMeetingId.current = meetingId;
            setReloadCompany(true);
            setOrgs([]);
            // User settings
            const result = await settingFmc.get();
            if (result.prompts != null) {
                for (let i = 0; i < result.prompts.length; i++) {
                    userPrompts.set(result.prompts[i].name, result.prompts[i].prompt);
                }
            }
            if (result.affinity_api_token != null && result.affinity_api_token.length > 0) {
                setAffinityIntegrationEnabled(true);
            }
            // Fetch the meeting
            const res_meeting = await meetingFmc.get(meetingId);
            console.log(res_meeting);
            setMeeting(res_meeting);
            setCompanyName(res_meeting.company_name);
            summaryEditor?.commands.setContent(res_meeting.summary); // Change the content of the editor when the prop is reused for a different meeting
            transcriptEditor?.commands.setContent(res_meeting.transcript);
            promptEditor?.commands.setContent(res_meeting.prompt);
            userPrompts.set("Previously used prompt", res_meeting.prompt);
            setMeetingTitle(res_meeting.title);
            setUserPromptsNameList([...userPrompts.keys()]);
            setSelectedUserPrompt("Previously used prompt");
            noteEditor?.commands.setContent(res_meeting.note);
            setDisplayDate((new Date(res_meeting.datetime).toDateString()) + " " + (new Date(res_meeting.datetime)).toLocaleTimeString())
            setCheckedPublishWithPersonalNote(res_meeting.publish_with_note ? res_meeting.publish_with_note : false);
            // Fetch the company if it's associated with one
            if (res_meeting.company_id.length > 0) {
                const crm_org = await crmFmc.get_org(res_meeting.company_id).catch((_err) => {""});
                setPublishPossible(true);
                setOrg(crm_org.name);
            } else {
                setOrg("");
            }
            // Reload the prop - end
            setReloadCompany(false);
            setReloadEditor(false);
            updateViews();
          } catch (error) {
            // Handle any errors here
            console.error('Error fetching setting:', error);
          }
        }
        fetchData();
      }, [meetingId]);

    useEffect(() => () => {
        // Save the meeting data -> Called when component unmount or the meeting prop changes
        const saveMeetingData = async (meeting: Meeting | null) => {
            if (meeting) {
              await updateMeeting(meeting);
            }
        };
        saveMeetingData(meeting); 
        updateViews();
      }, [meeting]);

    
    // Handle window closing
    windowTauri.getCurrent().listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async () => {
        console.log("Window closing ...")
        if (exit_called) {
            return;
        }
        exit_called = true;
        if (meeting) {
            await updateMeeting(meeting)
        }
        invoke('exit');
    })
    
    // Retranscript logic
    async function retranscript() {
        if (!meeting) {return;}
        let clonedMeeting = { ...meeting };
        let lang = language === "English" ? "En" : language === "Français" ? "Fr" : "Zh";
        notifications.show({
            title: 'Transcription started!',
            message: 'Watson will ping you when the transcription is done.',
            icon: <QuestionMarkIcon />,
            color: 'gray',
            withBorder: true,
            withCloseButton: true,
            autoClose: 4000,
        })
        await meetingFmc.increment_async_ops_count(clonedMeeting);
        try {
            updateViews(); // Update the meeting list - spiner on meeting list
            let m = await recorderFmc.transcribe(clonedMeeting, lang);
            let res_meeting = await meetingFmc.get(clonedMeeting.uuid);
            res_meeting.transcript = m.transcript;
            await meetingFmc.update(res_meeting); // Frontend update of transcript -> save
            // Update the meeting object in the frontend - if the page is on the same meeting
            if (clonedMeeting.uuid == mountedMeetingId.current) {
                console.log("Updating current meeting");
                setMeeting(res_meeting);
                meeting.transcript = res_meeting.transcript;
                transcriptEditor?.commands.setContent(res_meeting.transcript);
            }
            await meetingFmc.summarize(res_meeting); // Backend update of summary -> Pull if needed
            if (clonedMeeting.uuid == mountedMeetingId.current) {
                console.log("Updating current meeting summary");
                let res_meeting = await meetingFmc.get(meetingId);
                meeting.summary = res_meeting.summary;
                setMeeting(meeting);
                summaryEditor?.commands.setContent(meeting.summary);
                setReloadEditor(false);
            }
            await meetingFmc.decrement_async_ops_count(clonedMeeting);
            notifications.show({
                title: clonedMeeting.title,
                message: 'The new transcript and summary is ready.',
                withCloseButton: true,
                withBorder: true,
                autoClose: 4000,
            })        
        } catch {
            await meetingFmc.decrement_async_ops_count(clonedMeeting);   
        }
        updateViews(); // Update the meeting list - remove spiner on meeting list
    }

    // Component ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    if (transcriptEditorView) {
        return (
            <Stack>
                <Modal opened={openedTranscriptModal} onClose={closeTranscriptModal} title="Instruction for audio processing" size="md" withCloseButton={false}>
                    <Stack>
                    <Group>
                        <Text>Notice: This action will also resummarize the meeting with the last prompt used.</Text>
                        <Text>Select language: </Text>
                        <SegmentedControl 
                        fullWidth 
                        color={computedColorScheme === "light" ? 'var(--mantine-color-borealGreen-4)' : 'var(--mantine-color-borealGreen-5)'} 
                        defaultValue={language} 
                        onChange={(value) => {
                            setLanguage(value)
                        }
                        } 
                        data={['English', 'Français', '中文']} 
                        />
                    </Group>
                    <Group justify="center">
                        <Button 
                        variant="outline" 
                        leftSection={<FileTextIcon/>}
                        onClick={async (event) => {
                            event.preventDefault();
                            closeTranscriptModal();
                            await retranscript();
                            updateViews(); // Update the meeting list - handle the case where the meeting is not the current one
                        }}>
                            Start transcription
                        </Button>
                        <Button 
                            onClick={closeTranscriptModal} 
                            variant='light'
                            leftSection={<Cross1Icon/>}
                        >
                            Cancel
                        </Button>
                    </Group>
                    </Stack>
                </Modal>
                <Flex mih={50}
                    gap="md"
                    justify="flex-start"
                    align="center"
                    direction="row"
                    wrap="nowrap"
                >
                    <Button leftSection={<ArrowLeftIcon/>} variant="default" onClick={() => {setTranscriptEditorView(false)}}>
                        Edit Meeting
                    </Button>
                    <Button variant="default" onClick={openTranscriptModal} leftSection={<FileTextIcon/>}>
                        Restart Transcript
                    </Button>
                    <TextInput
                        w={5000}
                        leftSection={<HeadingIcon/>}
                        placeholder="Meeting Title"
                        value={meeting?.title}
                        disabled
                    /> 
                </Flex>
                <RichTextEditor editor={transcriptEditor}>
                    <RichTextEditor.Content />
                </RichTextEditor>
            </Stack>
        );
    }

    // Component ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    if (promptEditorView) {
        return (
            <Stack>
                <Modal opened={openedPromptModal} onClose={closePromptModal} title="Re-Summarize Transcript" size="md" withCloseButton={false}>
                    <Stack>
                    <Text>Notice: This will ignore the current summary and re-summarize the transcript.</Text>
                    <Group justify="center">
                        <Button 
                        variant="outline" 
                        leftSection={<Pencil1Icon/>}
                        onClick={async (event) => {
                            event.preventDefault();
                            setPromptEditorView(false);
                            if (meeting) {
                                let clonedMeeting = { ...meeting };
    
                                setReloadEditor(true);
                                // Save the prompt to the meeting object in the backend
                                await meetingFmc.update(clonedMeeting).catch((err) => {
                                    console.log(err)
                                });
                                // Summarize the meeting
                                await meetingFmc.summarize(clonedMeeting).catch((err) => {
                                    console.log(err)
                                });
                                // Update the meeting object in the frontend - if the page is on the same meeting
                                if (clonedMeeting.uuid == mountedMeetingId.current) {
                                    console.log("Updating current meeting");
                                    let res_meeting = await meetingFmc.get(meetingId);
                                    meeting.summary = res_meeting.summary;
                                    setMeeting(meeting);
                                    summaryEditor?.commands.setContent(meeting.summary);
                                    setReloadEditor(false);
                                } else {
                                    notifications.show({
                                        title: clonedMeeting.title,
                                        message: 'Summary updated!',
                                        withBorder: true,
                                        autoClose: 4000,
                                        withCloseButton: true,
                                    })
                                }
                            }
                            closePromptModal();
                            updateViews(); // Update the meeting list - handle the case where the meeting is not the current one
                        }}>
                            Re-Summarize Transcript
                        </Button>
                        <Button 
                            onClick={closePromptModal} 
                            variant='light'
                            leftSection={<Cross1Icon/>}
                        >
                            Cancel
                        </Button>
                    </Group>
                    </Stack>
                </Modal>
                <Flex mih={50}
                    gap="md"
                    justify="flex-start"
                    align="center"
                    direction="row"
                    wrap="nowrap"
                >
                    <Button leftSection={<ArrowLeftIcon/>} variant="default" onClick={() => {setPromptEditorView(false)}}> Edit Meeting </Button>
                    <Button variant="default" leftSection={<Pencil1Icon/>} onClick={openPromptModal}> Re-Summarize Transcript </Button>
                    <TextInput
                        w={5000}
                        leftSection={<HeadingIcon/>}
                        placeholder="Meeting Title"
                        value={meeting?.title}
                        disabled
                    /> 
                </Flex>
                <NativeSelect 
                    description="Select a prompt or write your own below"
                    value={selectedUserPrompt}
                    data={userPromptsNameList}
                    onChange={(event) => {
                        setSelectedUserPrompt(event.currentTarget.value);
                        let p = userPrompts.get(event.currentTarget.value)
                        if (p)
                            promptEditor?.commands.setContent(p);
                    }}
                />
                <RichTextEditor editor={promptEditor}>
                    <RichTextEditor.Content />
                </RichTextEditor>                
            </Stack>
        );
    }

    return (
        <Stack>        
         
        <Modal opened={deleteModalOpened} onClose={close} title="Are you sure you want to delete this meeting?" centered>
            <Center h = {50}>
                <Group>
                    <Button leftSection={<TrashIcon/> } variant="outline" color="red" onClick={async () => {
                    await meetingFmc.delete(meetingId).catch((err) => {
                        console.log(err)
                    });
                    updateViews();
                    window.location.href = "/";
                    }}>
                        Delete
                    </Button>
                </Group>
            </Center>

        </Modal>
        
        <Group align='flex-end' justify="space-between">
        
            {affinityIntegrationEnabled ? (
                <Group>
                    {reloadCompany ? ( <p>Loading ...</p> ) : (
                        <Autocomplete
                            w={220}
                            placeholder="Company"
                            description="Meeting with"
                            value={org}
                            onChange={async (event: string) => {
                                if (!meeting) {
                                    return
                                }
                                setOrg(event);
                                meeting.company_name = event;

                                let new_orgs = [];
                                // Remove current company attached to the meeting
                                if (event.length == 0) {
                                    meeting.company_id = "";
                                    meeting.company_name = "";
                                    setPublishPossible(false);
                                }
                                // Search
                                if (event.length > 3) {
                                    let orgsAffinity = (await search_org(event)).organizations;
                                    for (let i = 0; i < orgsAffinity.length; i++) {
                                        let org_name = orgsAffinity[i].name + " (" + orgsAffinity[i].domain + ")";
                                        new_orgs.push(org_name);
                                        orgsNameToId.set(org_name, orgsAffinity[i].id);
                                    }
                                    setOrgsNameToId(orgsNameToId);
                                    setOrgs(new_orgs);
                                }
                                // Set company to meeting
                                if (orgsNameToId.has(event)) {
                                    let company_id = orgsNameToId.get(event);
                                    if (company_id) {
                                        meeting.company_id = company_id.toString();
                                        meeting.company_name = event
                                        setPublishPossible(true);
                                    }
                                }
                            }}
                            data={orgs}
                        />
                    )}
                </Group>
            ) : (
                <Group>
                    <TextInput
                        w={220}
                        placeholder="Company"
                        description="Meeting with"
                        value={companyName}
                        onChange={async (event) => {
                            console.log(event.currentTarget.value);
                            setCompanyName(event.currentTarget.value);
                            if (meeting) {
                                meeting.company_name = event.currentTarget.value;
                                meeting.company_id = "";
                            }
                        }}
                    />
                </Group>
            )}
            

        <Flex mih={50} gap="md" justify="flex-start" align="flex-end" direction="row" wrap="wrap">
            <TextInput w={209} description="Recorded on" value={displayDate} disabled />
            <ActionIcon variant="outline" color="red" size="lg" onClick={openDeleteModal}>
                <TrashIcon/>
            </ActionIcon>
        </Flex>
        
        </Group>

        <TextInput
            leftSection={<HeadingIcon/>}
            placeholder="Meeting Title"
            value={meetingTitle}
            onChange={async (event) => {
                if (meeting) {
                    setMeetingTitle(event.currentTarget.value);
                    meeting.title = event.currentTarget.value;
                }
            }}
        />        

        <SegmentedControl
            onChange={setEditorSelector}
            data={[
                { value: 'Summary', label: 'Summary' },
                { value: 'Notes', label: 'Personal Notes' },
            ]}
        />

        {editorSelector == "Notes" ? (
            <Stack>
                <RichTextEditor editor={noteEditor}>
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

                    <RichTextEditor.Content/>
                </RichTextEditor>
            </Stack>

        ) : (
            reloadEditor ? (
                <p>Summarizing...</p>
            ) : (
                <Stack>

                <RichTextEditor editor={summaryEditor}>
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

                    <RichTextEditor.Content/>
                </RichTextEditor>

                <Group justify="space-between" mb={5} style={{paddingBottom: 20}}>
                    <Group>
                        <Button variant="light" leftSection={<MagicWandIcon/>} onClick={() => {setPromptEditorView(true);}}> Prompt </Button>
                        <Button variant="light" leftSection={<ReaderIcon/>} onClick={() => {setTranscriptEditorView(true);}}> Transcript </Button>
                    </Group>
                    { affinityIntegrationEnabled ? (
                        <Group>
                            <Checkbox
                                defaultChecked
                                label="Include Notes"
                                variant="outline"
                                checked={checkedPublishWithPersonalNote}
                                onChange={(event) => {
                                    if (!meeting) {return}
                                    meeting.publish_with_note = event.currentTarget.checked;
                                    setCheckedPublishWithPersonalNote(event.currentTarget.checked)
                                }}
                            />
                            { publishPossible ? (
                                meeting?.published ? (
                                    <Button 
                                    leftSection={<UploadIcon/>}
                                    variant="outline"
                                    onClick={
                                        async (event) => {
                                            event.preventDefault();
                                            if (meeting) {
                                                await updateMeeting(meeting);
                                                await crmFmc.publish(meeting);
                                                notifications.show({
                                                    title: 'Summary published!',
                                                    message: 'The summary was successfully published to Affinity.',
                                                    withBorder: true,
                                                    color: 'blue',
                                                    icon: <UploadIcon/>,
                                                    autoClose: 4000,
                                                    withCloseButton: true,
                                                })
                                            }
                                        }
                                    }>
                                        Re-upload to Affinity
                                    </Button>  
                                ) : (
                                <Button 
                                leftSection={<UploadIcon/>}
                                onClick={
                                    async (event) => {
                                        event.preventDefault();
                                        if (meeting) {
                                            await updateMeeting(meeting);
                                            await crmFmc.publish(meeting);
                                            meeting.published = true;
                                            await updateMeeting(meeting);
                                            notifications.show({
                                                title: 'Summary published to Affinity!',
                                                message: 'The summary was successfully published to Affinity.',
                                                withBorder: true,
                                                color: 'blue',
                                                icon: <UploadIcon/>,
                                                autoClose: 4000,
                                                withCloseButton: true,
                                            })
                                            updateViews();
                                        }
                                    }
                                }>
                                    Upload to Affinity
                                </Button>
                                )
                            ) : (
                                <Button disabled> Publish to Affinity </Button>
                            )}
                        </Group>
                    ) : (
                        <Group> 
                            <Button
                                leftSection={<CopyIcon/>}
                                variant="outline"
                                onClick={(event) => {
                                    event.preventDefault();
                                    if (meeting) {
                                        navigator.clipboard.writeText(meeting.summary);
                                    }
                                }}
                            > Copy to clipboard </Button>
                        </Group>
                    )}
                </Group>
            </Stack>
            )

        )}
        </Stack>
    );
}


