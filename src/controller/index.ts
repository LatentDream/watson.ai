import { Meeting } from '../bindings/index.js';
import { ipc_invoke } from '../ipc.js';
import { ModelMutateResultData } from '../bindings/ModelMutateResultData.js';
import { MeetingForUpdate } from '../bindings/MeetingForUpdate.js';
import { Setting } from '../bindings/Setting.js';
import { NewMeetingNote } from '../bindings/NewMeetingNote.js';
import { MeetingsRef } from '../bindings/MeetingsRef.js';
import { AvailableDevices } from '../bindings/AvailableDevices.js';
import { RecordingDevices } from '../bindings/RecordingDevices.js';


class MeetingModelController {

    async list(): Promise<MeetingsRef[]> {
        // prune the empty string
        return ipc_invoke(`list_meetings`).then(res => res.data);
      }

    async update(meeting: Meeting): Promise<ModelMutateResultData> {
      let meetingForUpdate: MeetingForUpdate = {meeting: meeting};
      return ipc_invoke(`update_meeting`, { id: meeting.uuid, data: meetingForUpdate } ).then(res => res.data);
    }

    async summarize(meeting: Meeting): Promise<ModelMutateResultData> { 
      return ipc_invoke(`async_summarize_meeting`, { id: meeting.uuid } ).then(res => res.data);
    }

    async improve_note(meeting: Meeting): Promise<ModelMutateResultData> {
      return ipc_invoke(`async_improve_note_meeting`, { id: meeting.uuid } ).then(res => res.data);
    }

    async get(id: string): Promise<Meeting> {
      return ipc_invoke(`get_meeting`, { id: id }).then(res => res.data);
    }

    async delete(id: string): Promise<ModelMutateResultData> {
      return ipc_invoke(`delete_meeting`, { id: id }).then(res => res.data);
    }

    async increment_async_ops_count(meeting: Meeting): Promise<ModelMutateResultData> {
      return ipc_invoke(`increment_async_ops_meeting`, { id: meeting.uuid }).then(res => res.data);
    }

    async decrement_async_ops_count(meeting: Meeting): Promise<ModelMutateResultData> {
      return ipc_invoke(`decrement_async_ops_meeting`, { id: meeting.uuid }).then(res => res.data);
    }

    async delete_all(): Promise<null> {
      return ipc_invoke(`delete_all_meeting`, {}).then(res => res.data);
    }

    async export_all(): Promise<string> {
      return ipc_invoke(`export_all_meeting`, {}).then(res => res.data);
    }

}

export const meetingFmc = new MeetingModelController();

class SettingModelController {

  async get(): Promise<Setting> {
    return ipc_invoke(`get_setting`, {}).then(res => res.data);
  }

  async update(setting: Setting): Promise<null> {
    return ipc_invoke(`update_setting`, { id: setting.uuid, data: setting }).then(res => res.data);
  }

  async open_data_folder(): Promise<null> {
    return ipc_invoke(`open_data_folder`, {}).then(res => res.data);
  }
}

export const settingFmc = new SettingModelController();


class RecordingModelController {

  async get_state(): Promise<string> {
    return ipc_invoke(`get_recording_state`, {}).then(res => res.data);
  }

  async get_recording_device_names(): Promise<RecordingDevices> {
    return ipc_invoke(`get_recording_device_names`, {}).then(res => res.data);
  }

  async start(inputDevice: string, outputDevice: string): Promise<string> {
    return ipc_invoke(`start_recording`, {recording_devices: {input_device_name: inputDevice, output_device_name: outputDevice}}).then(res => res.data)
  }

  async pause(): Promise<null> {
    return ipc_invoke(`pause_recording`, {}).then(res => res.data);
  }

  async resume(): Promise<null> {
    return ipc_invoke(`resume_recording`, {}).then(res => res.data);
  }

  async stop(): Promise<Meeting> { 
    return ipc_invoke(`stop_recording`, {}).then(res => res.data);
  }

  async transcribe(meeting: Meeting, language: String): Promise<Meeting> { 
    /* supported language: "En", "Fr" */
    return ipc_invoke(`transcribe_recording`, { path: meeting.audio_path, language: language}).then(
      res => {
        console.log("Transcript: " + res);
        let transcript = res.data
        meeting.transcript = transcript;
        return meeting;
      });
  }

  async get_available_audio_devices(): Promise<AvailableDevices> {
    return ipc_invoke(`get_available_audio_devices`, {}).then(res => res.data);
  }

}

export const recorderFmc = new RecordingModelController();


class CRMModelController {
  async search_orgs(query: string): Promise<any> {
    return ipc_invoke(`search_organizations_crm`, { query: query }).then(res => res.data);
  }

  async search_persons(query: string): Promise<any> {
    return ipc_invoke(`search_persons_crm`, { query: query }).then(res => res.data);
  }

  async publish(meeting: Meeting): Promise<any> {
    return ipc_invoke(`publish_summary_crm`, { id: meeting.uuid }).then(res => res.data);
  }

  async get_org(id: string): Promise<any> {
    return ipc_invoke(`get_organization_crm`, { id: id }).then(res => res.data);
  }

  async list_lists(): Promise<any> {
    return ipc_invoke(`list_lists_crm`, {}).then(res => res.data);
  }

}

export const crmFmc = new CRMModelController();


class SessionController {
  
  async get_new_meeting_note(): Promise<any> {
    return ipc_invoke(`get_new_meeting_note`, {}).then(res => res.data);
  }

  async set_new_meeting_note(new_meeting_note: NewMeetingNote): Promise<any> {
    return ipc_invoke(`set_new_meeting_note`, { id: "", data: new_meeting_note }).then(res => res.data);
  }

}

export const sessionFmc = new SessionController();
