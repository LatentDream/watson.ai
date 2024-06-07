import { useDisclosure } from '@mantine/hooks';
import { PasswordInput, Stack, Button, Textarea, NativeSelect, Fieldset, TextInput, ActionIcon, Flex, Modal, Group, Text, HoverCard, Select } from '@mantine/core';
import { crmFmc, meetingFmc, settingFmc } from '../controller';
import { Setting } from '../bindings/Setting';
import { useState, useEffect, useRef } from 'react';
import { invoke, window as windowTauri } from "@tauri-apps/api"
import { TauriEvent } from "@tauri-apps/api/event"
import { ModelTurbo } from '../bindings/ModelTurbo';
import { CommitIcon, Cross1Icon, ExternalLinkIcon, TrashIcon } from '@radix-ui/react-icons';
import { useAppContext } from '../AppContext';
import { notifications } from '@mantine/notifications';

let exit_called = false;


export default function Settings() {

    // @ts-ignore 
    const { views, updateViews } = useAppContext();
    const [visible, { toggle }] = useDisclosure(false);
    const [setting, setSetting] = useState<Setting | null>(null);
    const [crmName2Id, SetCrmName2Id] = useState<Map<string, string>>(new Map());

    // Handle window closing
    windowTauri.getCurrent().listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async () => {
      if (exit_called) {
          return;
      }
      exit_called = true;
      invoke('exit');
    })

    // Load settings on mount
    useEffect(() => {
      async function fetchData() {
        try {
          const result = await settingFmc.get();
          setSetting(result);
          setModel(result.default_model? result.default_model : "GPT3");
          if (result.prompts != null) {
            for (let i = 0; i < result.prompts.length; i++) {
              prompts.set(result.prompts[i].name, result.prompts[i].prompt);
              setPromptsNameList([...prompts.keys()]);
              setSelectedPrompt(result.prompts[i].name);
              setEditorPromptName(result.prompts[i].name);
              setEditorPrompt(result.prompts[i].prompt);
            }
          }
          if (result.affinity_api_token != null && result.affinity_api_token != "") {
            let lists = await crmFmc.list_lists();
            let list = null;
            crmName2Id.clear();
            let crmLists: string[] = [];
            for (let i = 0; i < lists.length; i++) {
              let listId = String(lists[i].id);
              crmLists.push(lists[i].name);
              crmName2Id.set(lists[i].name, listId);
              if (result.affinity_crm_list_id === listId) {
                list = lists[i].name;
              }
            }
            SetCrmName2Id(crmName2Id);
            setCrmLists(crmLists);
            setCrmList(list);
          }
        } catch (error) {
          // Handle any errors here
          console.error('Error fetching setting:', error);
        }
      }
      fetchData();
    }, []); // The empty array [] as the second argument means this effect will run once when the component mounts.
  
    // Save settings ->  when component unmount
    useEffect(() => () => {
      if (setting != null) {
        settingFmc.update(setting);
      }   
    }, [setting]);

    const ref_openai = useRef<HTMLInputElement>(null);
    const ref_assemblyai = useRef<HTMLInputElement>(null);
    const ref_affinity = useRef<HTMLInputElement>(null);
    const [model, setModel] = useState<string>("");
    const [editorPromptName, setEditorPromptName] = useState<string>("");
    const [editorPrompt, setEditorPrompt] = useState<string>("");
    const [selectedPrompt, setSelectedPrompt] = useState('');
    const [prompts, _setPrompts] = useState<Map<string, string>>(new Map());
    const [promptsNameList, setPromptsNameList] = useState<string[]>([...prompts.keys()]);
    const [openedDeleteModal, { open: openDeleteModal, close: closeDeleteModal }] = useDisclosure(false);
    const [openedExportModal, { open: openExportModal, close: closeExportModal }] = useDisclosure(false);
    const [openedOpenModal, { open: openOpenModal, close: closeOpenModal }] = useDisclosure(false);
    const [crmLists, setCrmLists] = useState<string[]>([]);
    const [crmList, setCrmList] = useState<string | null>(null);

    function savePrompts() {
      let promptList: {name: string, prompt: string}[] = [];
      prompts.forEach((value, key) => {
        promptList.push({name: key, prompt: value});
      });
      if (setting)
        setting.prompts = promptList;      
    }

    return (
      <Stack>
        <Modal opened={openedDeleteModal} onClose={closeDeleteModal} title="Delete all meetings" size="md" withCloseButton={false}>
          <Stack>
            <Text>Notice: This action cannot be undone. Are you sure you want to proceed?</Text>
            <Group justify="center">
              <Button 
                color="red"
                variant='outline'
                leftSection={<TrashIcon/>}
                onClick={async () => {
                  await meetingFmc.delete_all();
                  updateViews();
                  closeDeleteModal();
                  notifications.show({
                    title: 'Meetings Deleted',
                    color: 'red',
                    message: 'All meetings have been deleted',
                    icon: <TrashIcon/>,
                    autoClose: 4000,
                    withBorder: true,
                    withCloseButton: true,
                  });
                }}
              >
                Delete all meetings
              </Button>
              <Button 
                onClick={closeDeleteModal} 
                variant='light'
                leftSection={<Cross1Icon/>}
              >
                Cancel
              </Button>
            </Group>
          </Stack>
        </Modal>
        <Modal opened={openedExportModal} onClose={closeExportModal} title="Export all meetings" size="md" withCloseButton={false}>
          <Stack>
            <Text>Notice: This action may take some time to complete.</Text>
            <Group justify="center">
              <Button 
                variant='outline'
                leftSection={<CommitIcon/>}
                onClick={async () => {
                  let path = await meetingFmc.export_all();
                  closeExportModal();
                  notifications.show({
                    title: 'Export completed',
                    message: 'All meetings have been exported: ' + path,
                    icon: <CommitIcon/>,
                    autoClose: 4000,
                    withBorder: true,
                    withCloseButton: true,
                  });
                }}
              >Export</Button>
              <Button 
                onClick={closeExportModal} 
                variant='light'
                leftSection={<Cross1Icon/>}
              >
                Cancel
              </Button>
            </Group>
          </Stack>
        </Modal>
        <Modal opened={openedOpenModal} onClose={closeOpenModal} title="Open Backend folder" size="md" withCloseButton={false}>
          <Stack>
            <Text>Notice: Be careful when modifying files in this folder, it may break the application.</Text>
            <Group justify="center">
            <Button 
                  leftSection={<ExternalLinkIcon/>}
                  variant="outline"
                  color="yellow"
                  onClick={async () => {await settingFmc.open_data_folder(); closeOpenModal();}}
                >
                  Open
              </Button>
              <Button 
                onClick={closeOpenModal} 
                variant='light'
                leftSection={<Cross1Icon/>}
              >
                Cancel
              </Button>
            </Group>
          </Stack>
        </Modal>
        <Fieldset legend="Prompt Editor">
          <Flex mih={50} gap="md" justify="flex-start" align="flex-end" direction="row">
            <NativeSelect 
              w={4000}
              label="My prompts" 
              description="Select a prompt to edit it or write a new name to create a new prompt"
              value={selectedPrompt}
              data={promptsNameList}
              onClick={() => {
                setEditorPromptName(selectedPrompt);
                setEditorPrompt(prompts.get(selectedPrompt) as string);
              }}
              onChange={(event) => {
                setEditorPromptName(event.currentTarget.value);
                setEditorPrompt(prompts.get(event.currentTarget.value) as string);
                setSelectedPrompt(event.currentTarget.value);
              }}
            />
            <ActionIcon 
              variant="outline" 
              color="red" 
              size="lg"
              onClick={() => {
                prompts.delete(selectedPrompt);
                setPromptsNameList([...prompts.keys()]);
                setSelectedPrompt(promptsNameList[0] ? promptsNameList[0] : promptsNameList[0]);
                setEditorPromptName(selectedPrompt);
                setEditorPrompt(prompts.get(selectedPrompt) as string);
                savePrompts();
              }}
            >
              <TrashIcon/>
            </ActionIcon>
          </Flex>
          
          <Flex mih={50} gap="md" justify="flex-start" align="flex-end" direction="row">
            <TextInput 
              w={4000}
              placeholder="Action items" 
              label="Name" 
              value={editorPromptName}
              onChange={(event) => {
                setEditorPromptName(event.currentTarget.value)
                if (prompts.has(event.currentTarget.value)) {
                  setSelectedPrompt(event.currentTarget.value);
                } else {
                  setSelectedPrompt('');
                }
              }}
            />
            <Button 
                variant={prompts.has(editorPromptName) ? "outline" : "filled"} 
                onClick={() => {
                  prompts.set(editorPromptName, editorPrompt);
                  setPromptsNameList([...prompts.keys()]);
                  setSelectedPrompt(editorPromptName);
                  savePrompts();
                }}
              >
                {prompts.has(editorPromptName) ? "Update existing prompt" : "Add new prompt"}
              </Button>
          </Flex>

          <Textarea
              placeholder="Extract the action items from the transcript"
              label="Prompt"
              value={editorPrompt}
              onChange={(event) => {setEditorPrompt(event.currentTarget.value)}}
            />
        </Fieldset>

        <Fieldset legend="Model">
          <NativeSelect 
            label="Model used for summarization" 
            value={model} 
            onChange={(event) => {
              setModel(event.currentTarget.value)
              if (setting) {
                let selectedModel: ModelTurbo;
                selectedModel = model === "GPT4" ? "GPT3" : "GPT4";
                setting.default_model = selectedModel
              }
            }} 
            data={['GPT3', 'GPT4']} />
        </Fieldset>

        <Fieldset legend="API Keys">
          <PasswordInput
            ref={ref_openai}
            defaultValue={setting?.openai_api_token}
            label="OpenAI - API Key"
            description="Located at: https://platform.openai.com/account/api-keys"
            visible={visible}
            onVisibilityChange={toggle}
            onChange={(event) => {
              if (setting) {
                setting.openai_api_token = event.currentTarget.value;
              }
            }}
          />
          <PasswordInput
            ref={ref_assemblyai}
            defaultValue={setting?.assemblyai_api_token}
            label="AssemblyAI - API Key"
            description="Located at: www.assemblyai.com/app/account"
            visible={visible}
            onVisibilityChange={toggle}
            onChange={(event) => {
              if (setting) {
                setting.assemblyai_api_token = event.currentTarget.value;
              }
            }}
          />
        </Fieldset>

        <Fieldset legend="Affinity Integration">
          <PasswordInput
            ref={ref_affinity}
            defaultValue={setting?.affinity_api_token}
            label="Affinity - API Key"
            description="Located at: https://YOUR_ORG_NAME.affinity.co/settings/api"      
            placeholder="Enter an API key - Or leave blank to disable"    
            visible={visible}
            onVisibilityChange={toggle}
            onChange={(event) => {
              if (setting) {
                setting.affinity_api_token = event.currentTarget.value;
                console.log("Affinity API token: " + event.currentTarget.value);
              }
            }}
          />

          <Select
            label="Automatically add companies to a list"
            placeholder="Select a list - Or leave blank to disable"
            data={crmLists}
            value={crmList}
            onChange={(value) => {
              if (setting) {
                if (value) {
                  let listId = crmName2Id.get(value);
                  if (listId) {
                    setting.affinity_crm_list_id = listId;
                  }
                } else {
                  setting.affinity_crm_list_id = null;

                }                
                setCrmList(value);
              }
            }}
            clearable
          />

        </Fieldset>

        <Fieldset legend="Action">
          <Flex
            mih={50}
            gap="md"
            justify="center"
            align="center"
            direction="row"
            wrap="wrap"
          >
            <HoverCard width={280} shadow="md">
              <HoverCard.Target>
                <Button 
                  leftSection={<CommitIcon/>}
                  variant="outline"
                  onClick={openExportModal}
                >
                  Export all meetings
                </Button>
              </HoverCard.Target>
              <HoverCard.Dropdown>
                <Text size="sm">
                  Export all meetings in a zip file.
                </Text>
              </HoverCard.Dropdown>
            </HoverCard>
            
            <HoverCard width={280} shadow="md">
              <HoverCard.Target>
                <Button 
                  leftSection={<ExternalLinkIcon/>}
                  variant="outline"
                  color="yellow"
                  onClick={openOpenModal}
                >
                  Open backend folder
              </Button>
              </HoverCard.Target>
              <HoverCard.Dropdown>
                <Text size="sm">
                  Access data generated by the application.
                </Text>
              </HoverCard.Dropdown>
            </HoverCard>

            <HoverCard width={280} shadow="md">
              <HoverCard.Target>
                <Button 
                  leftSection={<TrashIcon/>}
                  variant="outline"
                  color="red"
                  onClick={openDeleteModal}
                >
                  Delete all meetings
                </Button>
              </HoverCard.Target>
              <HoverCard.Dropdown>
                <Text size="sm">
                  You might want to export all meetings before deleting them.
                </Text>
              </HoverCard.Dropdown>
            </HoverCard>

            
          </Flex>
        </Fieldset>
      </Stack>


    );
}