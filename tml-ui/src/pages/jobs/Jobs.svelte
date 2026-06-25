<script lang="ts">
  import {
    Loading,
    NotificationQueue,
    StructuredList,
    StructuredListBody,
    StructuredListCell,
    StructuredListHead,
    StructuredListRow,
  } from "carbon-components-svelte";
  import { apiClientExt } from "../../lib/api";
  import { onMount, tick } from "svelte";

  let queue: NotificationQueue;

  interface JobItem {
    id: number;
    job_type: string;
    status: string;
    success: boolean;
    created_at: string;
  }

  interface ReadAllJobResponse {
    items: JobItem[];
    next_cursor: number | null;
  }

  let items: JobItem[] = [];
  let nextCursor: number | null = null;
  let pageSize = 10;
  let isLoading = false;

  async function fetchPage() {
    isLoading = true;
    try {
      let url = `/mgmt/read_all_job?page_size=${pageSize}`;
      if (nextCursor) {
        url = url + `&cursor=${nextCursor}`;
      }
      const response = await apiClientExt.get<ReadAllJobResponse>(url);
      if (!response.success || !response.data) {
        throw new Error(response.message ?? "");
      }

      items = [...items, ...response.data.items];
      nextCursor = response.data.next_cursor;
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    } finally {
      isLoading = false;
    }
  }

  async function checkAndLoad(node: Element) {
    if (!node || isLoading || !nextCursor) return;
    const rect = node.getBoundingClientRect();
    const isVisible = rect.top < window.innerHeight && rect.bottom >= 0;
    if (!isVisible) return;
    await fetchPage();
    await tick();
    checkAndLoad(node);
  }

  function infiniteScroll(node: Element) {
    const observer = new IntersectionObserver(
      (entries) => {
        const first = entries[0];
        if (first.isIntersecting && !isLoading && nextCursor) {
          checkAndLoad(node);
        }
      },
      { threshold: 0.1 },
    );

    observer.observe(node);

    return {
      destroy() {
        observer.unobserve(node);
      },
    };
  }

  onMount(() => {
    fetchPage();
  });
</script>

<NotificationQueue bind:this={queue} />

<StructuredList flush>
  <StructuredListHead>
    <StructuredListRow head>
      <StructuredListCell head>Id</StructuredListCell>
      <StructuredListCell head>Type</StructuredListCell>
      <StructuredListCell head>Success</StructuredListCell>
      <StructuredListCell head>Status</StructuredListCell>
      <StructuredListCell head>Created at</StructuredListCell>
    </StructuredListRow>
  </StructuredListHead>
  <StructuredListBody>
    {#each items as item (item.id)}
      <StructuredListRow>
        <StructuredListCell>{item.id}</StructuredListCell>
        <StructuredListCell><strong>{item.job_type}</strong></StructuredListCell
        >
        <StructuredListCell>{item.success}</StructuredListCell>
        <StructuredListCell>{item.status}</StructuredListCell>
        <StructuredListCell>{item.created_at}</StructuredListCell>
      </StructuredListRow>
    {/each}
  </StructuredListBody>
</StructuredList>

{#if nextCursor}
  <div use:infiniteScroll class="loading-trigger">
    {#if isLoading}
      <Loading withOverlay={false} small />
    {/if}
  </div>
{:else}
  <p class="no-more">No more data!</p>
{/if}

<style>
  .loading-trigger {
    display: flex;
    justify-content: center;
    padding: 2rem;
    min-height: 50px;
  }
  .no-more {
    text-align: center;
    color: #8d8d8d;
    padding: 2rem;
  }
</style>
