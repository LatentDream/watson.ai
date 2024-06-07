// Create a new file for your context, e.g., AppContext.js
import { PlusIcon, GearIcon, ChatBubbleIcon, SymbolIcon } from "@radix-ui/react-icons";
import { createContext, useContext, useState } from 'react';
import Recording from "./view/recording";
import Setting from "./view/setting";
import { meetingFmc, recorderFmc } from "./controller";
import MeetingView from "./view/meeting";
import { MeetingsRef } from "./bindings/MeetingsRef";
import { MantineProvider } from '@mantine/core';
import { Notifications } from '@mantine/notifications';

// Create a context
const AppContext = createContext(null);

export interface View {
    path: string, 
    title: string, 
    exact: boolean, 
    component: any,
    icon: any,
    display: boolean
    currently_processing: boolean
}

const defaultViews: View[] = [ 
  {
      path: "/",
      title: "New Recording",
      exact: true,
      component: Recording,
      icon: <PlusIcon/>,
      display: false,
      currently_processing: false
    },
    {
      path: "/settings",
      title: "Settings",
      exact: true,
      component: Setting,
      icon: <GearIcon/>,
      display: false,
      currently_processing: false
  }
]

// @ts-ignore
export function AppProvider({ children }) {
  const [views, setViews] = useState([...defaultViews]);
  const [notifRecording, setNotifRecording] = useState(false);

  const notifyChangeInRecordingState = async () => {
    const result = await recorderFmc.get_state();
    console.log("Recording state: " + result);
    if (result == "Recording" || result == "Paused") {
      setNotifRecording(true);
    } else {
      setNotifRecording(false);
    }
  }

  console.log("Init app provider");
  console.log(views);

  const updateViews = async () => {  
    let newViews = [...defaultViews];
    let meetings: MeetingsRef[] = await meetingFmc.list();
    // Sort base on date
    meetings.sort((a: MeetingsRef, b: MeetingsRef) => {
      return (new Date(a.datetime)) > (new Date(b.datetime)) ? 1 : -1;
    });
    if (meetings) {
        meetings.forEach((meeting: MeetingsRef) => {
          console.log("Title: " + meeting.title);
          newViews.splice(1, 0, {
            path: meeting.uuid,
            title: meeting.title,
            exact: true,
            component: MeetingView,
            icon: meeting.number_ops > 0 ? <SymbolIcon/> : <ChatBubbleIcon/>,
            display: true,
            currently_processing: meeting.number_ops > 0 ? true : false
          })
        });
      };
      console.log("Added meeting");
    console.log(views);
    setViews(newViews);
  };

  return (
    <>
      <MantineProvider 
        defaultColorScheme="dark"
        theme={{
          primaryColor: 'borealGreen',
          colors: {
            'borealGreen': ['#E1F0E5', '#BCC9C5', '#768C84', '#536E63', '#425F53', '#304F42', '#234B3B', '#154734', '#173D32', '#18332F'],
            'purple': ['#F0E1F0', '#C9BCC9', '#8C768C', '#6E536E', '#5F425F', '#4F304F', '#4B234B', '#431543', '#3D173D', '#331833'],
          },
        }}
      >
      <Notifications />
      {/* @ts-ignore */}
      <AppContext.Provider value={{ views, updateViews, notifRecording, notifyChangeInRecordingState }}>    
          {children}
      </AppContext.Provider>
    </MantineProvider>
    </>
  );
}

// Create a custom hook to use the context
export function useAppContext() {
  return useContext(AppContext);
}
