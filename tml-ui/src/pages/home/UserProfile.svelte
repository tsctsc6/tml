<script lang="ts">
  import {
    Button,
    Column,
    Grid,
    NotificationQueue,
    Row,
  } from "carbon-components-svelte";
  import { apiClientExt } from "../../lib/api";
  import { onMount } from "svelte";
  import type { UserProfileType } from "./userProfileType";

  let queue: NotificationQueue;

  interface Props {
    onLogout: () => void;
  }
  let { onLogout }: Props = $props();

  let userProfile: UserProfileType | null = $state(null);

  onMount(async () => {
    try {
      const response = await apiClientExt.get<UserProfileType>(
        "/auth/read_user_info",
      );
      if (!response.success) {
        throw new Error(response.message ?? "");
      }
      userProfile = response.data;
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    }
  });

  async function handleLogout(e: Event) {
    if (typeof window === "undefined") {
      return;
    }
    localStorage.removeItem("token");
    onLogout();
  }
</script>

<NotificationQueue bind:this={queue} />

<Grid>
  <Row>
    <Column>
      {#if userProfile}
        <h2>Hello! {userProfile.username}</h2>{:else}{/if}
    </Column>
  </Row>
  <Row
    ><Column>
      <Button on:click={handleLogout}>Logout</Button>
    </Column></Row
  >
</Grid>
