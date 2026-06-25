<script lang="ts">
  import {
    Column,
    Content,
    Grid,
    Header,
    Row,
    SideNav,
    SideNavItems,
    SideNavLink,
    SkipToContent,
  } from "carbon-components-svelte";
  import Home from "./pages/home/Home.svelte";
  import type { Component } from "svelte";
  import Users from "./pages/users/Users.svelte";
  import Jobs from "./pages/jobs/Jobs.svelte";
  import Storages from "./pages/storages/Storages.svelte";

  interface NavItem {
    id: string;
    text: string;
    component: Component;
  }

  const navItems: NavItem[] = [
    { id: "home", text: "Home", component: Home },
    { id: "users", text: "Users", component: Users },
    { id: "storages", text: "Storages", component: Storages },
    { id: "jobs", text: "Jobs", component: Jobs },
  ];

  let currentView = "home";
  $: activeItem = navItems.find((item) => item.id === currentView);
  $: activeComponent = activeItem ? activeItem.component : null;

  let isSideNavOpen = false;
</script>

<Header
  persistentHamburgerMenu={true}
  companyName="TML"
  platformName="Management"
  bind:isSideNavOpen
>
  <svelte:fragment slot="skipToContent"><SkipToContent /></svelte:fragment>
</Header>

<SideNav bind:isOpen={isSideNavOpen}>
  <SideNavItems>
    {#each navItems as item}<SideNavLink
        text={item.text}
        isSelected={currentView === item.id}
        on:click={(e) => {
          e.preventDefault();
          currentView = item.id;
        }}
      />{/each}
  </SideNavItems>
</SideNav>

<Content>
  {#if activeComponent}
    <svelte:component this={activeComponent} />
  {:else}
    <p>Page not found</p>
  {/if}
</Content>
