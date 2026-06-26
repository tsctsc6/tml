<script lang="ts">
  import {
    Button,
    Column,
    Dropdown,
    Grid,
    Loading,
    Modal,
    NotificationQueue,
    NumberInput,
    Row,
    StructuredList,
    StructuredListBody,
    StructuredListCell,
    StructuredListHead,
    StructuredListRow,
    Tag,
    TextInput,
    Tile,
    Toolbar,
    ToolbarContent,
  } from "carbon-components-svelte";
  import TrashCan from "carbon-icons-svelte/lib/TrashCan.svelte";
  import Add from "carbon-icons-svelte/lib/Add.svelte";
  import { apiClientExt } from "../../lib/api";
  import { onMount, tick } from "svelte";

  let queue: NotificationQueue;

  /// Read all
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

  let rows: JobItem[] = [];
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

      rows = [...rows, ...response.data.items];
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
    // wait svelte update DOM
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

  /// Read details
  interface ReadJobResponse {
    id: number;
    job_type: string;
    job_args: object;
    status: string;
    description: string;
    error_message: string;
    success: boolean;
    created_by_id: number;
    created_at: string;
    completed_at: string;
  }

  let isDetailsModalOpen = false;
  let selectedJobId = 0;
  let jobDetails: ReadJobResponse | null = null;

  function handleRowClick(item: JobItem) {
    selectedJobId = item.id;
    isDetailsModalOpen = true;
  }

  async function readJobDetails(id: number) {
    try {
      const response = await apiClientExt.get<ReadJobResponse>(
        `/mgmt/read_job?id=${id}`,
      );
      if (!response.success || !response.data) {
        throw new Error(response.message ?? "");
      }
      jobDetails = response.data;
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    }
  }

  function statusToTagType(
    status: string,
  ):
    | "red"
    | "magenta"
    | "purple"
    | "blue"
    | "cyan"
    | "teal"
    | "green"
    | "gray"
    | "cool-gray"
    | "warm-gray"
    | "high-contrast"
    | "outline"
    | undefined {
    switch (status) {
      case "WaitingStart":
        return "outline";
      case "Running":
        return "blue";
      case "Completed":
        return "green";
      default:
        return "gray";
    }
  }

  /// Delete
  interface DeleteJobRequest {
    id: number;
  }

  interface DeleteJobResponse {}

  let itemToDeleteId = 0;
  let isDeleteModalOpen = false;
  let isDeleting = false;

  function triggerDelete(event: PointerEvent, item: JobItem): void {
    event.stopPropagation();
    itemToDeleteId = item.id;
    isDeleteModalOpen = true;
  }

  async function confirmDelete() {
    if (itemToDeleteId == 0) return;
    isDeleting = true;

    try {
      const deleteJobRequest: DeleteJobRequest = {
        id: itemToDeleteId,
      };
      const response = await apiClientExt.post<DeleteJobResponse>(
        "/mgmt/delete_job",
        deleteJobRequest,
      );
      if (!response.success || !response.data) {
        throw new Error(response.message ?? "");
      }
      rows = rows.filter((item) => item.id !== itemToDeleteId);
      isDeleteModalOpen = false;
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    } finally {
      isDeleting = false;
      itemToDeleteId = 0;
    }
  }

  /// Create
  type CreateJobRequest =
    | {
        job_type: string;
        description: string;
      }
    | {
        job_type: string;
        description: string;
        job_args: ScanIncrementalArgs;
      };

  interface ScanIncrementalArgs {
    storage_id: number;
  }

  interface CreateJobResponse {
    id: number;
  }

  let isCreateModalOpen = false;
  let jobTypeSelectedId = 0;
  let createJobRequest: CreateJobRequest = { job_type: "", description: "" };
  let scanIncrementalArgs: ScanIncrementalArgs = { storage_id: 0 };
  let isCreating = false;

  let jobTypeMap = [
    { id: 0, text: "scan_incremental" },
    { id: 1, text: "build_index" },
    { id: 2, text: "update_index" },
    { id: 3, text: "delete_index" },
    { id: 4, text: "rebuild_index" },
  ];

  function triggerCreate() {
    isCreateModalOpen = true;
    jobTypeSelectedId = 0;
    createJobRequest = { job_type: "", description: "" };
  }

  async function confirmCreate() {
    console.log(jobTypeSelectedId);
    createJobRequest.job_type =
      jobTypeMap.find((x) => x.id === jobTypeSelectedId)?.text ?? "";
    switch (jobTypeSelectedId) {
      case 0:
        createJobRequest = {
          ...createJobRequest,
          job_args: { ...scanIncrementalArgs },
        };
        break;
      default:
        break;
    }
    try {
      const response = await apiClientExt.post<CreateJobResponse>(
        "/mgmt/create_job",
        createJobRequest,
      );
      if (!response.success || !response.data) {
        throw new Error(response.message ?? "");
      }
      rows = [];
      nextCursor = null;
      fetchPage();
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    } finally {
      isCreating = false;
      isCreateModalOpen = false;
    }
  }
</script>

<NotificationQueue bind:this={queue} />

<h4>Jobs</h4>

<Toolbar>
  <ToolbarContent>
    <Button icon={Add} iconDescription="Add" on:click={triggerCreate} />
  </ToolbarContent>
</Toolbar>

<StructuredList flush selection>
  <StructuredListHead>
    <StructuredListRow head>
      <StructuredListCell head>Id</StructuredListCell>
      <StructuredListCell head>Type</StructuredListCell>
      <StructuredListCell head>Success</StructuredListCell>
      <StructuredListCell head>Status</StructuredListCell>
      <StructuredListCell head>Created at</StructuredListCell>
      <StructuredListCell head></StructuredListCell>
    </StructuredListRow>
  </StructuredListHead>
  <StructuredListBody>
    {#each rows as item (item.id)}
      <StructuredListRow on:click={() => handleRowClick(item)}>
        <StructuredListCell>{item.id}</StructuredListCell>
        <StructuredListCell>
          <strong>{item.job_type}</strong>
        </StructuredListCell>
        <StructuredListCell>{item.success}</StructuredListCell>
        <StructuredListCell>{item.status}</StructuredListCell>
        <StructuredListCell>{item.created_at}</StructuredListCell>
        <StructuredListCell class="action-cell">
          <Button
            kind="danger-tertiary"
            size="small"
            iconDescription="Delete"
            icon={TrashCan}
            on:click={(e) => triggerDelete(e, item)}
          />
        </StructuredListCell>
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

<Modal
  bind:open={isDetailsModalOpen}
  modalHeading="Job details"
  primaryButtonText="Ok"
  on:click:button--primary={() => (isDetailsModalOpen = false)}
  on:open={() => {
    readJobDetails(selectedJobId);
  }}
  on:close={() => {
    selectedJobId = 0;
    jobDetails = null;
  }}
>
  {#if jobDetails}
    <Tile class="detail-tile">
      <Grid padding>
        <Row class="detail-header-row">
          <Column sm={4} md={4} lg={4} class="info-block">
            <span class="meta-label">Id</span>
            <code class="meta-value-code">{jobDetails.id}</code>
          </Column>
          <Column sm={4} md={4} lg={4} class="align-right">
            <span class="meta-label">Status</span>
            <div class="tag-wrapper">
              <Tag type={statusToTagType(jobDetails.status)}>
                {jobDetails.status}
              </Tag>
            </div>
          </Column>
          <Column sm={4} md={4} lg={4} class="align-right">
            <span class="meta-label">Success</span>
            <div class="tag-wrapper">
              <Tag type={jobDetails.success ? "green" : "red"}>
                {jobDetails.success}
              </Tag>
            </div>
          </Column>
          <Column sm={4} md={4} lg={4} class="info-block">
            <span class="meta-label">Created by</span>
            <span class="meta-value">{jobDetails.created_by_id}</span>
          </Column>
        </Row>

        <hr class="carbon-divider" />

        <Row>
          <Column sm={4} md={4} class="info-block">
            <span class="meta-label">Type</span>
            <h3 class="detail-title">{jobDetails.job_type}</h3>
          </Column>
          <Column sm={4} md={4} class="info-block">
            <span class="meta-label">Arguments</span>
            <span class="meta-value">
              {JSON.stringify(jobDetails.job_args)}
            </span>
          </Column>
        </Row>

        <Row class="margin-top-md">
          <Column sm={4} md={4} class="info-block">
            <span class="meta-label">Description</span>
            <span class="meta-value">{jobDetails.description}</span>
          </Column>

          <Column sm={4} md={4} class="info-block">
            <span class="meta-label">Error Message</span>
            <span class="meta-value">{jobDetails.error_message}</span>
          </Column>
        </Row>

        <Row class="margin-top-md">
          <Column sm={4} md={4} class="info-block">
            <span class="meta-label">Created At</span>
            <span class="meta-value">{jobDetails.created_at}</span>
          </Column>

          <Column sm={4} md={4} class="info-block">
            <span class="meta-label">Completed At</span>
            <span class="meta-value">{jobDetails.completed_at}</span>
          </Column>
        </Row>
      </Grid>
    </Tile>
  {:else}
    <p>Loading...</p>
  {/if}
</Modal>

<Modal
  danger
  bind:open={isDeleteModalOpen}
  modalHeading="Are you sure to delete?"
  primaryButtonText={isDeleting ? "Deleting..." : "Delete"}
  secondaryButtonText="Cancael"
  primaryButtonDisabled={isDeleting}
  on:click:button--primary={confirmDelete}
  on:click:button--secondary={() => (isDeleteModalOpen = false)}
  on:close={() => (isDeleteModalOpen = false)}
>
  <p>
    Are you sure to delete job <strong>{itemToDeleteId}</strong> ?
  </p>
</Modal>

<Modal
  bind:open={isCreateModalOpen}
  modalHeading="Create job"
  primaryButtonText={isCreating ? "Creating" : "Create"}
  secondaryButtonText="Cancel"
  primaryButtonDisabled={isCreating}
  on:click:button--primary={confirmCreate}
  on:click:button--secondary={() => (isCreateModalOpen = false)}
  on:close={() => {
    createJobRequest = { job_type: "", description: "" };
  }}
  ><div class="edit-form">
    <Dropdown
      labelText="Job Type"
      bind:selectedId={jobTypeSelectedId}
      items={jobTypeMap}
    />
  </div>
  {#if jobTypeSelectedId === 0}
    <NumberInput
      labelText="Storage Id"
      bind:value={scanIncrementalArgs.storage_id}
      placeholder="Storage Id"
    />
  {:else}{/if}
  <div class="edit-form">
    <TextInput
      labelText="Description"
      bind:value={createJobRequest.description}
      placeholder="Description"
    />
  </div></Modal
>

<style>
  .edit-form :global(.bx--form-item) {
    margin-bottom: 1.25rem;
  }

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

  :global(.detail-tile) {
    background-color: var(--cds-layer-01, #f4f4f4) !important;
    border-radius: 4px;
    border-left: 4px solid var(--cds-interactive-01, #0f62fe);
    padding: 0 !important;
  }

  .detail-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--cds-text-primary, #161616);
    margin-top: 0.25rem;
  }

  .meta-label {
    display: block;
    font-size: 0.75rem;
    color: var(--cds-text-secondary, #525252);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .meta-value {
    display: block;
    font-size: 0.9rem;
    color: var(--cds-text-primary, #161616);
    margin-top: 0.25rem;
  }

  .meta-value-code {
    display: inline-block;
    background: var(--cds-field-01, #e0e0e0);
    padding: 0.15rem 0.4rem;
    font-family: monospace;
    font-size: 0.85rem;
    border-radius: 2px;
    margin-top: 0.25rem;
  }

  .tag-wrapper {
    margin-top: 0.25rem;
  }

  .carbon-divider {
    border: none;
    border-top: 1px solid var(--cds-border-subtle, #e0e0e0);
    margin: 1.25rem 0;
  }
</style>
