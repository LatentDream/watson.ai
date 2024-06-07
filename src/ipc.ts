import { invoke } from "@tauri-apps/api";
import { notifications } from '@mantine/notifications';
import { listen } from '@tauri-apps/api/event';

// TODO: Fix double rendering of Recording view
var HISTORY: Map<string, any> = new Map();


/** 
 * Small wrapper on top of tauri api invoke for a light abstraction. 
 */

export async function ipc_invoke(method: string, params?: object): Promise<any> {
	const response: any = await invoke(method, { params });
		if (response.error != null) {
			console.log('ERROR - ipc_invoke - ipc_invoke error', response);
			// @ts-ignore
			if (!HISTORY.has(response.error.message) || new Date().getTime() - HISTORY.get(response.error.message) > 300) {
			notifications.show({
				title: 'Error!',
				message: response.error.message,
				autoClose: 4000,
				withCloseButton: true,
				color: 'red',
				withBorder: true,
			});
			// @ts-ignore
			HISTORY.set(response.error.message, new Date().getTime());
		}
		throw new Error(response);
	} else {
		return response.result;
	}
}


async function listenToEvent() {
	// @ts-ignore
	const _unlisten = await listen('ERROR', (event) => {
	  	console.log('ERROR - Event', event);
		console.log('Hello: ' + new Date().getTime());
		// @ts-ignore
		if (!HISTORY.has(event.payload.message) || new Date().getTime() - HISTORY.get(event.payload.message) > 300) {
			notifications.show({
				title: 'Critical Error!',
				// @ts-ignore
				message: event.payload.message,
				autoClose: 30000,
				withCloseButton: true,
				color: 'red',
				withBorder: true,
			});
			// @ts-ignore
			HISTORY.set(event.payload.message, new Date().getTime());
		}
	});
	// @ts-ignore
	const _warnUnlistent = await listen('WARNING', (event) => {
	  	console.log('WARNING - Event', event);
		// @ts-ignore
		if (!HISTORY.has(event.payload.message) || new Date().getTime() - HISTORY.get(event.payload.message) > 300) {
			notifications.show({
				title: 'Warning!',
				// @ts-ignore
				message: event.payload.message,
				autoClose: 30000,
				withCloseButton: true,
				color: 'yellow',
				withBorder: true,
			});
			// @ts-ignore
			HISTORY.set(event.payload.message, new Date().getTime());
		}
	});
}
  
listenToEvent();
