import '../module-resolver-helper'

import { assert } from 'chai'
import {
  connectionCreateInviterNull,
  createConnectionInviterInvited,
  createConnectionInviterRequested,
  dataConnectionCreate
} from 'helpers/entities'
import {
  INVITE_ACCEPTED_MESSAGE,
  PROTOCOL_TYPE_ARIES_STRICT
} from 'helpers/test-constants'
import { initVcxTestMode, shouldThrow, sleep } from 'helpers/utils'
import { Connection, StateType, VCXCode, VCXMock, VCXMockMessage } from 'src'

describe('Connection:', () => {
  before(() => initVcxTestMode(PROTOCOL_TYPE_ARIES_STRICT))

  describe('create:', () => {
    it('success', async () => {
      await connectionCreateInviterNull()
    })

    it('success: parallel', async () => {
      const numConnections = 50
      const data = dataConnectionCreate()
      await Promise.all(new Array(numConnections).fill(0).map(() => connectionCreateInviterNull(data)))
    })
  })

  describe('connect:', () => {
    it('success: without phone', async () => {
      const connection = await connectionCreateInviterNull()
      const inviteDetails = await connection.connect({ data: '{}' })
      assert.notEqual(inviteDetails, '')
    })

    it('throws: not initialized', async () => {
      const connection = new (Connection as any)()
      const err = await shouldThrow(async () => connection.connect({ data: '{}' }))
      assert.equal(err.vcxCode, VCXCode.INVALID_CONNECTION_HANDLE)
    })
  })

  // todo : restore for aries
  describe('sendMessage:', () => {
    it.skip('success: sends message', async () => {
      const connection = await connectionCreateInviterNull()
      await connection.connect({ data: '{"connection_type":"QR"}' })
      const error = await shouldThrow(() => connection.sendMessage({ msg: 'msg', type: 'msg', title: 'title' }))
      assert.equal(error.vcxCode, VCXCode.NOT_READY)
    })
  })

  describe('signData:', () => {
    it('success: signs data', async () => {
      const connection = await connectionCreateInviterNull()
      await connection.connect({ data: '{}' })
      const signature = await connection.signData(new Buffer('random string'))
      assert(signature)
    })
  })

  describe('verifySignature', () => {
    it('success: verifies the signature', async () => {
      const connection = await createConnectionInviterRequested()
      const valid = await connection.verifySignature({data: new Buffer('random string'),
        signature: new Buffer('random string')})
      assert(valid)
    })
  })

  describe('serialize:', () => {
    it('success', async () => {
      const connection = await connectionCreateInviterNull()
      const serialized = await connection.serialize()
      assert.ok(serialized)
      assert.property(serialized, 'version')
      assert.property(serialized, 'data')
      assert.property(serialized, 'state')
      assert.property(serialized.data, 'source_id')
      assert.property(serialized.data, 'pw_did')
      assert.property(serialized.data, 'pw_verkey')
      assert.property(serialized.data, 'endpoint')
      assert.property(serialized.data, 'their_pw_did')
      assert.property(serialized.data, 'their_pw_verkey')
      const { data, version } = serialized
      assert.ok(data)
      assert.ok(version)
      assert.equal(data.source_id, connection.sourceId)
      assert.equal(data.state, StateType.Initialized)
    })

    // TODO: restore for aries
    it.skip('throws: not initialized', async () => {
      const connection = new (Connection as any)()
      const error = await shouldThrow(() => connection.serialize())
      assert.equal(error.vcxCode, VCXCode.INVALID_CONNECTION_HANDLE)
    })

    // TODO: Is this op supported in 3.0?
    it.skip('throws: connection deleted', async () => {
      const connection = await connectionCreateInviterNull()
      await connection.connect({ data: '{"connection_type":"QR"}' })
      await connection.delete()
      const error = await shouldThrow(() => connection.serialize())
      assert.equal(error.vcxCode, VCXCode.INVALID_CONNECTION_HANDLE)
    })
  })

  describe('deserialize:', () => {
    it('success', async () => {
      const connection1 = await connectionCreateInviterNull()
      const data1 = await connection1.serialize()
      const connection2 = await Connection.deserialize(data1)
      assert.equal(connection2.sourceId, connection1.sourceId)
      const data2 = await connection2.serialize()
      assert.deepEqual(data1, data2)
    })

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(async () => Connection.deserialize({ data:
          { source_id: 'Invalid' } } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON)
    })
  })

  describe('updateState:', () => {
    it(`returns ${StateType.None}: not initialized`, async () => {
      const connection = new (Connection as any)()
      const state1 = await connection.updateState()
      const state2 = await connection.getState()
      assert.equal(state1, state2)
      assert.equal(state2, StateType.None)
    })

    it(`returns ${StateType.Initialized}: not connected`, async () => {
      const connection = await connectionCreateInviterNull({ id: 'alice' })
      await connection.updateState()
      assert.equal(await connection.getState(), StateType.Initialized)
    })

    // todo : restore for aries
    it.skip(`returns ${StateType.OfferSent}: connected`, async () => {
      const connection = await createConnectionInviterRequested()
      VCXMock.setVcxMock(VCXMockMessage.AcceptInvite) // todo: must return Aries mock data
      await connection.updateState()
      assert.equal(await connection.getState(), StateType.Accepted)
    })

    // todo : restore for aries
    it.skip(`returns ${StateType.Accepted}: mocked accepted`, async () => {
      const connection = await createConnectionInviterRequested()
      VCXMock.setVcxMock(VCXMockMessage.GetMessages)
      await connection.updateState()
      assert.equal(await connection.getState(), StateType.Accepted)
    })

    // todo : restore for aries
    it.skip(`returns ${StateType.Accepted}: mocked accepted`, async () => {
      const connection = await createConnectionInviterRequested()
      await connection.updateStateWithMessage(INVITE_ACCEPTED_MESSAGE)
      assert.equal(await connection.getState(), StateType.Accepted)
    })

    // todo : restore for aries
    it.skip(`returns ${StateType.Accepted}: mocked accepted in parallel`, async () => {
      const numConnections = 50
      const interval = 50
      const sleepTime = 100
      const connectionsWithTimers = await Promise.all(new Array(numConnections).fill(0).map(async () => {
        const connection = await connectionCreateInviterNull()
        const timer = setInterval(() => connection.updateState(), interval)
        return { connection, timer }
      }))
      let cond = false
      while (cond) {
        const states = await Promise.all(connectionsWithTimers.map(({ connection }) => connection.getState()))
        cond = states.every((state) => state === StateType.Accepted)
        VCXMock.setVcxMock(VCXMockMessage.GetMessages)
        await sleep(sleepTime)
      }
      connectionsWithTimers.forEach(({ timer }) => clearInterval(timer))
    })
  })

  describe('inviteDetails:', () => {
    it('success', async () => {
      const connection = await createConnectionInviterInvited()
      const details = await connection.inviteDetails(true)
      const parsedInvitation = JSON.parse(details)
      assert.isString(parsedInvitation['@id'])
      assert.equal(parsedInvitation['@type'], 'did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation')
      assert.isString(parsedInvitation.label)
      assert.isArray(parsedInvitation.recipientKeys)
      assert.equal(parsedInvitation.recipientKeys.length, 1)
      assert.isArray(parsedInvitation.routingKeys)
      assert.equal(parsedInvitation.routingKeys.length, 2)
      assert.equal(parsedInvitation.serviceEndpoint, 'http://127.0.0.1:8080/agency/msg')
    })
  })

  describe('sendPing:', () => {
    it('success: send ping', async () => {
      const connection = await connectionCreateInviterNull()
      await connection.sendPing('ping')
    })
  })

  describe('sendDiscoveryFeatures:', () => {
    it('success: send discovery features', async () => {
      const connection = await connectionCreateInviterNull()
      await connection.sendDiscoveryFeatures('*', 'comment')
    })
  })
})
