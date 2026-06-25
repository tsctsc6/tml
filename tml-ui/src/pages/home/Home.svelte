<script lang="ts">
  import {
    Column,
    Grid,
    NotificationQueue,
    Row,
  } from "carbon-components-svelte";
  import Login from "./Login.svelte";
  import UserProfile from "./UserProfile.svelte";
  import { onMount } from "svelte";
  import { apiClientExt } from "../../lib/api";

  interface ReadUserInfoResponse {
    username: string;
    password: string;
  }

  let isLoggedIn = false;
  let queue: NotificationQueue;

  onMount(async () => {
    try {
      const response = await apiClientExt.get<ReadUserInfoResponse>(
        "/auth/read_user_info",
      );
      if (!response.success) {
        throw new Error(response.message ?? "");
      }
      isLoggedIn = true;
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    }
  });
</script>

<NotificationQueue bind:this={queue} />

<Grid>
  <Row>
    <Column><h1>Home</h1></Column>
  </Row>
  <Row>
    <Column>
      {#if isLoggedIn}<UserProfile />{:else}<Login />{/if}
    </Column>
  </Row>
</Grid>
