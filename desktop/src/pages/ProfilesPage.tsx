import { useEffect, useState } from "react";
import { api } from "../api";
import type { DeviceSnapshot, Profile, ProfileInput, ProfileList } from "../contracts";

interface Props {
  snapshot: DeviceSnapshot;
  onSnapshot: (value: DeviceSnapshot) => void;
  onError: (error: unknown) => void;
}

const ANC_MAXIMUM = 10;
const newProfile: ProfileInput = {
  name: "Custom", cnc_level: 0, auto_cnc: false,
  spatial: "off", wind_block: false, anc_toggle: true,
};

export function profileAncStrength(cncLevel: number) {
  return ANC_MAXIMUM - Math.max(0, Math.min(ANC_MAXIMUM, cncLevel));
}

export function deviceCncLevel(ancStrength: number) {
  return ANC_MAXIMUM - Math.max(0, Math.min(ANC_MAXIMUM, ancStrength));
}

export function ProfilesPage({ snapshot, onSnapshot, onError }: Props) {
  const [list, setList] = useState<ProfileList | null>(null);
  const [draft, setDraft] = useState<ProfileInput | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let active = true;
    setDraft(null);
    setBusy(true);
    void api.getProfiles(snapshot.generation)
      .then((value) => active && setList(value))
      .catch(onError)
      .finally(() => active && setBusy(false));
    return () => { active = false; };
  }, [snapshot.generation, onError]);

  function edit(profile: Profile) {
    if (!profile.editable) return;
    setDraft({
      profile_id: profile.profile_id, name: profile.name,
      cnc_level: profile.cnc_level, auto_cnc: profile.auto_cnc,
      spatial: profile.spatial, wind_block: profile.wind_block,
      anc_toggle: profile.anc_toggle,
    });
  }

  async function save() {
    if (!draft) return;
    setBusy(true);
    try {
      setList(await api.setProfile(snapshot.generation, draft));
      setDraft(null);
    } catch (error) {
      onError(error);
    } finally {
      setBusy(false);
    }
  }

  async function remove(profile: Profile) {
    if (!profile.editable || !window.confirm(`Delete profile “${profile.name}”?`)) return;
    setBusy(true);
    try {
      setList(await api.deleteProfile(snapshot.generation, profile.profile_id));
      if (draft?.profile_id === profile.profile_id) setDraft(null);
    } catch (error) {
      onError(error);
    } finally {
      setBusy(false);
    }
  }

  async function activate(profile: Profile) {
    setDraft(null);
    setBusy(true);
    try {
      onSnapshot(await api.setNoiseControl({
        generation: snapshot.generation, kind: "mode", value: profile.name,
      }));
    } catch (error) {
      onError(error);
    } finally {
      setBusy(false);
    }
  }

  const customCount = list?.profiles.filter((profile) => profile.editable).length ?? 0;
  const maximum = snapshot.capabilities.profiles?.maximum ?? 0;
  const canCreate = list !== null && customCount < maximum;

  return <div className="page-grid">
    <section className="card" aria-labelledby="profiles-title">
      <div className="card__heading">
        <div><p className="eyebrow">Listening presets</p><h2 id="profiles-title">Profiles</h2></div>
        <div className="button-row">
          {busy && <span role="status">Working…</span>}
          <button className="secondary" type="button" disabled={busy || !canCreate} onClick={() => setDraft({ ...newProfile })}>New profile</button>
        </div>
      </div>
      <ul className="profile-list">{list?.profiles.map((profile) => {
        const anc = profile.anc_toggle ? `ANC ${profileAncStrength(profile.cnc_level)}/10` : "ANC off";
        return <li key={profile.profile_id} className={snapshot.values.profile_id === profile.profile_id ? "active" : ""}>
          <button className="profile-main secondary" type="button" disabled={busy} onClick={() => void activate(profile)}>
            <strong>{profile.name}</strong><small>{anc} · {profile.spatial}</small>
          </button>
          {profile.editable && <>
            <button className="icon-button secondary" aria-label={`Edit ${profile.name}`} type="button" disabled={busy} onClick={() => edit(profile)}>Edit</button>
            <button className="icon-button danger-button" aria-label={`Delete ${profile.name}`} type="button" disabled={busy} onClick={() => void remove(profile)}>Delete</button>
          </>}
        </li>;
      })}</ul>
      {!busy && list?.profiles.length === 0 && <p className="muted">No profiles were reported.</p>}
    </section>

    <section className="card" aria-labelledby="profile-editor-title">
      <h2 id="profile-editor-title">{draft ? (draft.profile_id ? "Edit profile" : "New profile") : "Profile details"}</h2>
      {!draft ? <p className="muted">Select a profile to activate it. Use Edit for a custom profile or New profile to change settings.</p> : <>
        <div className="form-grid">
          <label>Name<input maxLength={64} value={draft.name} onChange={(event) => setDraft({ ...draft, name: event.currentTarget.value })} /></label>
          <label>
            ANC strength <output>{profileAncStrength(draft.cnc_level)}/10</output>
            <input aria-label="Profile ANC strength" type="range" min="0" max="10" value={profileAncStrength(draft.cnc_level)} disabled={!draft.anc_toggle} onChange={(event) => setDraft({ ...draft, cnc_level: deviceCncLevel(Number(event.currentTarget.value)) })} />
            <span className="range-ends"><small>Awareness</small><small>Maximum ANC</small></span>
          </label>
          <label>Spatial audio<select value={draft.spatial} onChange={(event) => setDraft({ ...draft, spatial: event.currentTarget.value as ProfileInput["spatial"] })}><option value="off">Off</option><option value="room">Room</option><option value="motion">Motion</option></select></label>
          <label className="check-label"><input type="checkbox" checked={draft.anc_toggle} onChange={(event) => setDraft({ ...draft, anc_toggle: event.currentTarget.checked })} /> Use ANC in this profile</label>
          <label className="check-label"><input type="checkbox" checked={draft.auto_cnc} disabled={!draft.anc_toggle} onChange={(event) => setDraft({ ...draft, auto_cnc: event.currentTarget.checked })} /> Adjust ANC automatically</label>
          <label className="check-label"><input type="checkbox" checked={draft.wind_block} onChange={(event) => setDraft({ ...draft, wind_block: event.currentTarget.checked })} /> Wind block</label>
          <p className="muted">Awareness (0/10) is different from turning ANC off. Wind block can reduce the audible effect of ANC strength changes.</p>
        </div>
        <div className="button-row">
          <button type="button" disabled={busy || !draft.name.trim()} onClick={() => void save()}>{draft.profile_id ? "Save changes" : "Create profile"}</button>
          <button className="secondary" type="button" disabled={busy} onClick={() => setDraft(null)}>Cancel</button>
        </div>
      </>}
    </section>
  </div>;
}
