import { useEffect, useState } from "react";
import { AppShell, Group, Burger, Text, ActionIcon, Divider, Image } from '@mantine/core';
import { PlusIcon, SunIcon, MoonIcon, GearIcon } from "@radix-ui/react-icons";
import { ScrollArea, ThemeIcon, useMantineColorScheme, useComputedColorScheme } from "@mantine/core";
import { MemoryRouter, NavLink, Route, Routes } from "react-router-dom";
import { View, useAppContext } from "./AppContext";
import { useDisclosure } from '@mantine/hooks';
import { notifications } from "@mantine/notifications";
import React from "react";
import iconApp from "./assets/icon.png";


function App() {

  // @ts-ignore 
  const { views, updateViews, notifRecording, notifyChangeInRecordingState } = useAppContext();
  const { setColorScheme } = useMantineColorScheme();
  const computedColorScheme = useComputedColorScheme('light', { getInitialValueInEffect: true });
  const [opened, { toggle }] = useDisclosure();
  const [active, setActive] = useState("/");
  const [height, setHeight] = useState(window.innerHeight - 60);

  React.useEffect(() => {
      window.addEventListener("resize", () => {setHeight(window.innerHeight - 60)});
      return () => window.removeEventListener("resize", () => {setHeight(window.innerHeight - 60)});
  });

  useEffect(() => {
    updateViews();
  }, []);
   
  return (
    <MemoryRouter>
      <AppShell
        header={{ height: 60 }}
        navbar={{ width: 200, breakpoint: 'sm', collapsed: { mobile: !opened }}}
        padding="md"
      >
        <AppShell.Header>
          <Group h="100%" px="md" justify="space-between">
            <Burger opened={opened} onClick={toggle}  hiddenFrom='sm' size="sm"/>
            { notifRecording ? (
              <Group>
                <Image src={iconApp} w={36}/> 
                <Text color="red">Watson is currently listening ...</Text>
              </Group>
            ) : 
            (
              <Group> 
                <Image src={iconApp} w={36}/> 
                <Text> Watson is ready to help</Text>
              </Group>
            )}
            <div style={{marginLeft: "auto"}}>
                <ActionIcon 
                  variant="default" 
                  onClick={() => setColorScheme(computedColorScheme === 'light' ? 'dark' : 'light')}
                  size={30}
                >
                  {computedColorScheme === "dark" ? <SunIcon/> : <MoonIcon/>}
              </ActionIcon>
            </div>
          </Group>
        </AppShell.Header>
        <AppShell.Navbar p="md" w={200} h={height}>
          <AppShell.Section>
            <NavLink 
              to={"/"} 
              onClick={() => {
                toggle(); 
                setActive("/"); 
                console.log("active: " + active);
              }}
              style={{ 
                color: "/" === active ? (
                  'var(--mantine-color-borealGreen-4)'
                ) : (
                  computedColorScheme === "light" ? 'var(--mantine-color-dark-10)' : 'var(--mantine-color-gray-1)'
                ),
                textDecoration: 'none', 
              }} 
            >
              <Group> 
                <ThemeIcon
                variant={"/" === active ? "filled" : "outline"}
                color={
                      "/" === active ? (
                        computedColorScheme === "light" ? 'var(--mantine-color-borealGreen-2)' : 'var(--mantine-color-borealGreen-5)'
                      ) : (
                      computedColorScheme === "light" ? 'var(--mantine-color-borealGreen-4)' : 'var(--mantine-color-borealGreen-5)'
                      )
                    }
                  >
                  <PlusIcon/>
                </ThemeIcon>
              <Text>New Recording</Text></Group>
            </NavLink>       
          </AppShell.Section>
          <Divider my="sm" />
          <AppShell.Section grow  component={ScrollArea}>
          {
              views.map((view: View, index: any) => 
              view.display ? (
                  <NavLink 
                    to={view.currently_processing ? active : view.path} 
                    key={index} 
                    onClick={() => {
                        if (view.currently_processing) {
                          notifications.show ({
                            title: "Processing ...",
                            message: "This meeting is currently being processed. Please wait until it is finished.",
                            autoClose: 4000,
                            color: "yellow",
                            withCloseButton: true,
                            withBorder: true,
                          })
                        } else {
                          toggle(); 
                          setActive(view.path);
                        }
                    }}
                    color="gray"
                    style={{ 
                      color: view.path === active ? (
                        'var(--mantine-color-borealGreen-4)'
                      ) : (
                        computedColorScheme === "light" ? 'var(--mantine-color-dark-10)' : 'var(--mantine-color-gray-1)'
                      ),
                      textDecoration: 'none',  
                    }} 
                  >
                    <Group h={45}> 
                      <ThemeIcon 
                      variant={view.path === active ? "filled" : "outline"}
                      color={
                        view.path === active ? (
                          computedColorScheme === "light" ? 'var(--mantine-color-borealGreen-2)' : 'var(--mantine-color-borealGreen-5)'
                        ) : (
                        computedColorScheme === "light" ? 'var(--mantine-color-borealGreen-4)' : 'var(--mantine-color-borealGreen-5)'
                        )
                      }
                      >
                        {view.icon}
                      </ThemeIcon>
                      <Text>{view.title.substring(0, 14)}</Text>
                    </Group>
                  </NavLink> 
                ) : null
              )
            }
          </AppShell.Section>
          <Divider my="sm" />
          <AppShell.Section>
          <NavLink 
            to={"settings"} 
            onClick={() => {toggle(); setActive("settings")}}  
            color="gray"
            style={{ 
              color: "settings" === active ? (
                'var(--mantine-color-borealGreen-4)'
              ) : (
                computedColorScheme === "light" ? 'var(--mantine-color-dark-10)' : 'var(--mantine-color-gray-1)'
              ),
              textDecoration: 'none', 
            }}
          >
                <Group> 
                  <ThemeIcon 
                    variant={"settings" === active ? "filled" : "outline"}
                    color={
                      "settings" === active ? (
                        computedColorScheme === "light" ? 'var(--mantine-color-borealGreen-2)' : 'var(--mantine-color-borealGreen-5)'
                      ) : (
                      computedColorScheme === "light" ? 'var(--mantine-color-borealGreen-4)' : 'var(--mantine-color-borealGreen-5)'
                      )
                    }>
                  <GearIcon/>
                  </ThemeIcon>
                <Text>Settings</Text></Group>
              </NavLink>
          </AppShell.Section>
        </AppShell.Navbar>
        <AppShell.Main>
          <Routes>
            {
              views.map((view: View, index: any) => 
                <Route key={index} path={view.path} element={ <view.component/> }/>
              )
            }
          </Routes>

        </AppShell.Main>
      </AppShell>
    </MemoryRouter>
  );

}

export default App;
